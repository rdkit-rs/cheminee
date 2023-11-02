#!/usr/bin/env ruby
require 'cheminee'

configuration = Cheminee::Configuration.new()
# configuration.host = "localhost:3000"
# configuration.scheme = "http"
configuration.host = "cheminee.scientist.com"
configuration.scheme = "https"

api_client = Cheminee::ApiClient.new(configuration)
default_api = Cheminee::DefaultApi.new(api_client)

default_api.v1_indexes_index_post("meepity-beepity", "descriptor_v1", sort_by: "exactmw") rescue nil

structures = File.read("structures").split("\n")

passes = 5
docs = []

for i in (1..passes)
  for structure in structures
    docs << {smile: structure, extra_data: {smile_again: structure, notice: "we're on pass #{i}"}}
  end
end

bulk_request = Cheminee::BulkRequest.build_from_hash(docs: docs)
result = default_api.v1_indexes_index_bulk_index_post("meepity-beepity", bulk_request)
