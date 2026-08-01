//! Scaffold networks: the hierarchy of ring systems and linkers a molecule reduces to,
//! computed by RDKit's `rdScaffoldNetwork`.
//!
//! Prior art for this class of hierarchy, though not the algorithm RDKit implements:
//! Wilkens, Janes & Su, "HierS: hierarchical scaffold clustering using topological
//! chemical graphs", J. Med. Chem. 2005, 48(9), 3182-93, doi:10.1021/jm049032d. HierS
//! enumerates ring-delimited substructures directly, where RDKit applies a bond breaking
//! reaction and also emits the generic and attachment point scaffolds seen in the node
//! flags below, which HierS has no equivalent of.
//!
//! Not to be confused with [`crate::search::scaffold_search`], which matches against a
//! static precomputed dictionary of scaffolds to accelerate tantivy queries.

use crate::search::compound_processing::standardize_smiles;
use poem_openapi_derive::Object;
use rdkit::{scaffold_network_for_mol, ROMol, ScaffoldNetworkEdgeType, ScaffoldNetworkParams};

/// Molecules bigger than this are rejected before RDKit is handed anything, because the
/// cost of enumerating a scaffold hierarchy climbs with the number of ring systems and
/// there is no way to interrupt RDKit once it is running.
pub const DEFAULT_MAX_ATOMS: u32 = 150;

/// A network larger than this is reported as an error rather than returned, which keeps a
/// single pathological input from dominating a batch response.
pub const DEFAULT_MAX_NODES: u32 = 5_000;

/// Knobs for one scaffold network computation. `None` everywhere means RDKit's own
/// defaults plus the limits above.
#[derive(Debug, Default, Clone)]
pub struct ScaffoldNetworkOptions {
    pub include_generic_scaffolds: Option<bool>,
    pub include_generic_bond_scaffolds: Option<bool>,
    pub include_scaffolds_with_attachments: Option<bool>,
    pub include_scaffolds_without_attachments: Option<bool>,
    pub keep_only_first_fragment: Option<bool>,
    pub prune_before_fragmenting: Option<bool>,
    pub flatten_isotopes: Option<bool>,
    pub flatten_chirality: Option<bool>,
    pub flatten_keep_largest: Option<bool>,
    pub collect_mol_counts: Option<bool>,
    pub bond_breaker_smarts: Option<Vec<String>>,
    pub standardize: Option<bool>,
    pub max_atoms: Option<u32>,
    pub max_nodes: Option<u32>,
}

impl ScaffoldNetworkOptions {
    fn max_atoms(&self) -> u32 {
        self.max_atoms.unwrap_or(DEFAULT_MAX_ATOMS)
    }

    fn max_nodes(&self) -> u32 {
        self.max_nodes.unwrap_or(DEFAULT_MAX_NODES)
    }

    fn rdkit_params(&self) -> ScaffoldNetworkParams {
        let mut params = match &self.bond_breaker_smarts {
            Some(smarts) => ScaffoldNetworkParams::with_bond_breakers(smarts),
            None => ScaffoldNetworkParams::default(),
        };

        if let Some(what) = self.include_generic_scaffolds {
            params.set_include_generic_scaffolds(what);
        }
        if let Some(what) = self.include_generic_bond_scaffolds {
            params.set_include_generic_bond_scaffolds(what);
        }
        if let Some(what) = self.include_scaffolds_with_attachments {
            params.set_include_scaffolds_with_attachments(what);
        }
        if let Some(what) = self.include_scaffolds_without_attachments {
            params.set_include_scaffolds_without_attachments(what);
        }
        if let Some(what) = self.keep_only_first_fragment {
            params.set_keep_only_first_fragment(what);
        }
        if let Some(what) = self.prune_before_fragmenting {
            params.set_prune_before_fragmenting(what);
        }
        if let Some(what) = self.flatten_isotopes {
            params.set_flatten_isotopes(what);
        }
        if let Some(what) = self.flatten_chirality {
            params.set_flatten_chirality(what);
        }
        if let Some(what) = self.flatten_keep_largest {
            params.set_flatten_keep_largest(what);
        }
        if let Some(what) = self.collect_mol_counts {
            params.set_collect_mol_counts(what);
        }

        params
    }
}

#[derive(Object, Debug, Clone, PartialEq)]
pub struct ScaffoldNetworkNode {
    /// Canonical SMILES for the scaffold. A molecule with no rings at all reduces to an
    /// empty scaffold, so this is legitimately the empty string for those.
    pub scaffold_smiles: String,
    /// Every atom in this scaffold has been replaced by a dummy atom.
    pub is_generic: bool,
    /// This scaffold still carries the attachment points where it was cut away from its
    /// parent.
    pub has_attachments: bool,
    /// How many times this scaffold was reached while walking the hierarchy.
    pub count: u32,
    /// How many input molecules this scaffold was found in. Absent when
    /// `collect_mol_counts` was turned off.
    #[oai(skip_serializing_if_is_none)]
    pub mol_count: Option<u32>,
}

/// One "is a more specific version of" link, always pointing from the child up to its
/// more general parent.
#[derive(Object, Debug, Clone, PartialEq)]
pub struct ScaffoldNetworkEdge {
    /// Index into `nodes` of the more general scaffold.
    pub parent_idx: u32,
    /// Index into `nodes` of the more specific scaffold.
    pub child_idx: u32,
    /// Canonical SMILES of `nodes[parent_idx]`, repeated here so an edge stands on its own.
    pub parent_smiles: String,
    /// Canonical SMILES of `nodes[child_idx]`, repeated here so an edge stands on its own.
    pub child_smiles: String,
    /// What RDKit did to get from the child to the parent: `Fragment`, `Generic`,
    /// `GenericBond`, `RemoveAttachment` or `Initialize`.
    pub edge_type: String,
}

#[derive(Object, Debug, Clone, PartialEq)]
pub struct ScaffoldNetworkResult {
    /// The SMILES as submitted, so results can be lined up with inputs.
    pub smiles: String,
    /// The SMILES actually fragmented, after standardization.
    #[oai(skip_serializing_if_is_none)]
    pub standardized_smiles: Option<String>,
    pub nodes: Vec<ScaffoldNetworkNode>,
    pub edges: Vec<ScaffoldNetworkEdge>,
    /// Set when this molecule could not be processed. The other fields are empty and the
    /// rest of the batch is unaffected.
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

impl ScaffoldNetworkResult {
    fn failed(smiles: &str, error: impl std::fmt::Display) -> Self {
        ScaffoldNetworkResult {
            smiles: smiles.to_string(),
            standardized_smiles: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            error: Some(error.to_string()),
        }
    }
}

/// Build the scaffold hierarchy for a single SMILES. Every failure mode — an unparseable
/// SMILES, a molecule too big to be worth fragmenting, a network that came out too large,
/// or an exception from RDKit itself — comes back as `Err` for the caller to report
/// per-molecule rather than failing the whole batch.
pub fn scaffold_network_for_smiles(
    smiles: &str,
    options: &ScaffoldNetworkOptions,
) -> eyre::Result<ScaffoldNetworkResult> {
    let standardize = options.standardize.unwrap_or(true);

    let romol = match standardize {
        true => standardize_smiles(smiles, false)?,
        false => ROMol::from_smiles(smiles).map_err(|e| eyre::eyre!("{}", e))?,
    };

    // RDKit runs to completion once it starts and cannot be interrupted, so the only real
    // protection against a pathological input is to refuse it up front
    let num_atoms = romol.num_atoms(true);
    if num_atoms > options.max_atoms() {
        return Err(eyre::eyre!(
            "molecule has {num_atoms} atoms, more than the {} allowed",
            options.max_atoms()
        ));
    }

    let standardized_smiles = romol.as_smiles();
    let network = scaffold_network_for_mol(&romol, &options.rdkit_params())
        .map_err(|e| eyre::eyre!("{}", e))?;

    if network.nodes.len() > options.max_nodes() as usize {
        return Err(eyre::eyre!(
            "scaffold network has {} nodes, more than the {} allowed",
            network.nodes.len(),
            options.max_nodes()
        ));
    }

    let attachment_flags = attachment_flags(&network);

    let nodes = network
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| ScaffoldNetworkNode {
            scaffold_smiles: node.scaffold_smiles.clone(),
            is_generic: attachment_flags[idx].is_generic,
            has_attachments: attachment_flags[idx].has_attachments,
            count: node.count,
            mol_count: node.mol_count,
        })
        .collect::<Vec<_>>();

    let edges = network
        .edges
        .iter()
        .map(|edge| ScaffoldNetworkEdge {
            parent_idx: edge.parent_idx() as u32,
            child_idx: edge.child_idx() as u32,
            parent_smiles: network.nodes[edge.parent_idx()].scaffold_smiles.clone(),
            child_smiles: network.nodes[edge.child_idx()].scaffold_smiles.clone(),
            edge_type: edge.edge_type.to_string(),
        })
        .collect::<Vec<_>>();

    Ok(ScaffoldNetworkResult {
        smiles: smiles.to_string(),
        standardized_smiles: Some(standardized_smiles),
        nodes,
        edges,
        error: None,
    })
}

#[derive(Default, Clone, Copy)]
struct NodeFlags {
    is_generic: bool,
    has_attachments: bool,
}

/// Work out which nodes are dummy-atom scaffolds and which still carry attachment points,
/// using the edges RDKit already labelled rather than re-parsing the scaffold SMILES.
/// Re-parsing would mean handing RDKit a generic SMILES like `*1:*:*:*:*:*:1`, and the
/// property-cache invariants it trips on there abort the process instead of raising.
///
/// RDKit only ever reaches a generic scaffold through a `Generic`/`GenericBond` edge, and
/// only ever strips attachment points through a `RemoveAttachment` edge, so:
///
///   * a node is generic exactly when a generic edge points at it
///   * a node has attachments exactly when a `RemoveAttachment` edge leaves it, or when it
///     is the generic twin of a node that does
///
/// This needs two passes, not one: RDKit emits a node's `Generic` edge before its
/// `RemoveAttachment` edge, so propagating during the first pass would read the attachment
/// flag before it had been set.
fn attachment_flags(network: &rdkit::ScaffoldNetwork) -> Vec<NodeFlags> {
    let mut flags = vec![NodeFlags::default(); network.nodes.len()];

    for edge in &network.edges {
        match edge.edge_type {
            ScaffoldNetworkEdgeType::RemoveAttachment => {
                flags[edge.child_idx()].has_attachments = true
            }
            ScaffoldNetworkEdgeType::Generic | ScaffoldNetworkEdgeType::GenericBond => {
                flags[edge.parent_idx()].is_generic = true
            }
            _ => (),
        }
    }

    for edge in &network.edges {
        if matches!(
            edge.edge_type,
            ScaffoldNetworkEdgeType::Generic | ScaffoldNetworkEdgeType::GenericBond
        ) {
            // a generic scaffold keeps whatever attachment points its concrete twin had,
            // and the same generic can be reached from more than one twin
            flags[edge.parent_idx()].has_attachments |= flags[edge.child_idx()].has_attachments;
        }
    }

    flags
}

/// Run a batch of SMILES through [`scaffold_network_for_smiles`], turning per-molecule
/// failures into per-molecule errors so one bad input cannot sink the batch.
pub fn scaffold_networks_for_smiles(
    smiles_vec: &[String],
    options: &ScaffoldNetworkOptions,
) -> Vec<ScaffoldNetworkResult> {
    use rayon::prelude::*;

    smiles_vec
        .par_iter()
        .map(
            |smiles| match scaffold_network_for_smiles(smiles, options) {
                Ok(result) => result,
                Err(e) => ScaffoldNetworkResult::failed(smiles, e),
            },
        )
        .collect::<Vec<_>>()
}
