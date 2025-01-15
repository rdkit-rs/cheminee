#!/usr/bin/env ruby
require 'cheminee'

configuration = Cheminee::Configuration.new()
configuration.host = "localhost:4001"
configuration.scheme = "http"
configuration.timeout = 300 # 5 minutes

smiles_raw_data = File.read(File.join(File.dirname(__FILE__), '../../../assets/standardized_scaffolds_20240405.json'))
smiles_data = smiles_raw_data.lines.collect{|l| JSON.parse(l) }
smiles_data = smiles_data * 50

api_client = Cheminee::ApiClient.new(configuration)
default_api = Cheminee::DefaultApi.new(api_client)

default_api.v1_indexes_index_post("meepity-beepity", "descriptor_v1", sort_by: "exactmw") rescue nil

smiles_data.each_slice(10_000).each do |chunk|
  docs = chunk.map{|smiles| {smiles: smiles["smiles"], extra_data: {compound_id: 123} } }
  bulk_request = Cheminee::BulkRequest.build_from_hash(docs: docs)
  result = default_api.v1_indexes_index_bulk_index_post("meepity-beepity", bulk_request)
end
