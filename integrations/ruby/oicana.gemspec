# frozen_string_literal: true

require_relative "lib/oicana/version"

Gem::Specification.new do |spec|
  spec.name = "oicana"
  spec.version = Oicana::VERSION
  spec.authors = ["Oicana"]
  spec.email = ["hello@oicana.com"]
  spec.summary = "Generate typeset PDFs in Ruby from Typst templates, in process. No headless browser, no per-document fees."
  spec.homepage = "https://oicana.com"
  spec.license = "PolyForm-Noncommercial-1.0.0"
  spec.metadata = {
    "source_code_uri" => "https://github.com/oicana/oicana/tree/main/integrations/ruby",
    "documentation_uri" => "https://oicana.com/docs",
    "rubygems_mfa_required" => "true"
  }
  spec.required_ruby_version = ">= 3.2"

  spec.files = Dir["lib/**/*.rb", "README.md", "LICENSE.md"]
  # Platform gems carry the native library, for example: OICANA_PLATFORM=x86_64-linux-gnu gem build
  if ENV["OICANA_PLATFORM"]
    spec.platform = ENV["OICANA_PLATFORM"]
    spec.files += Dir["lib/oicana/native/*"]
  end

  spec.add_dependency "ffi", "~> 1.16"
end
