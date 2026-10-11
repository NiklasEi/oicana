# frozen_string_literal: true

require "json"

module Oicana
  # The value of a blob input. Metadata is passed to the template along with the data
  # and must be a Hash, or nil for none.
  BlobInput = Data.define(:data, :metadata) do
    def initialize(data:, metadata: nil)
      super
    end
  end

  # Limits for reading a packed template. nil uses the default of Oicana.
  ZipLimits = Data.define(:max_entries, :max_total_decompressed_bytes) do
    def initialize(max_entries: nil, max_total_decompressed_bytes: nil)
      super
    end
  end

  # The size of a page in points.
  PageSize = Data.define(:width, :height)

  # A font face registered by the host. path is nil for fonts registered from memory.
  RegisteredFont = Data.define(:family, :path)

  # The result of Template.export_once. warnings is nil when there are none.
  ExportOnceResult = Data.define(:document, :warnings)

  # A template's manifest.
  Manifest = Data.define(:package, :oicana)

  # The Typst package a template is.
  PackageInfo = Data.define(:name, :version, :entrypoint, :authors, :license, :description, :homepage,
                            :repository)

  # The Oicana configuration of a template. inputs holds JsonInputDefinition and
  # BlobInputDefinition values in manifest order.
  OicanaConfig = Data.define(:manifest_version, :inputs, :validate_json_inputs_by_default, :export, :fonts)

  # An input taking a JSON value. default, development and schema name files in the template.
  JsonInputDefinition = Data.define(:key, :required, :default, :development, :schema, :validate)

  # An input taking arbitrary bytes. default and development are BlobFallback values or nil.
  BlobInputDefinition = Data.define(:key, :required, :default, :development)

  # A blob from the template, used when no value is supplied.
  BlobFallback = Data.define(:file, :meta)

  # How compiled documents are exported.
  ExportConfig = Data.define(:pdf)

  # How documents are exported to PDF.
  PdfExportConfig = Data.define(:standards, :tagged)

  # Fonts a template expects from its host.
  FontConfig = Data.define(:require)

  # @api private
  # Conversions between the public types and the generated bindings.
  module Convert
    MODES = {
      development: OicanaUniffi::CompilationMode::DEVELOPMENT,
      production: OicanaUniffi::CompilationMode::PRODUCTION
    }.freeze

    module_function

    def mode(mode)
      MODES.fetch(mode) { raise ArgumentError, "unknown compilation mode #{mode.inspect}, use :development or :production" }
    end

    def format(format, pixels_per_pt)
      case format
      when :pdf then OicanaUniffi::ExportFormat::PDF.new
      when :svg then OicanaUniffi::ExportFormat::SVG.new
      when :png then OicanaUniffi::ExportFormat::PNG.new(Float(pixels_per_pt))
      else raise ArgumentError, "unknown export format #{format.inspect}, use :pdf, :png or :svg"
      end
    end

    def json_inputs(inputs)
      inputs.to_h do |key, value|
        [key.to_s, value.is_a?(String) ? utf8(value, key) : JSON.generate(value)]
      end
    end

    # Binary strings, as read by File.binread or from HTTP bodies, are taken to be UTF-8.
    def utf8(json, key)
      json = json.encoding == Encoding::BINARY ? json.dup.force_encoding(Encoding::UTF_8) : json.encode(Encoding::UTF_8)
      raise InvalidInputError, "the JSON input #{key.to_s.inspect} is not valid UTF-8" unless json.valid_encoding?

      json
    rescue EncodingError => e
      raise InvalidInputError, "the JSON input #{key.to_s.inspect} cannot be converted to UTF-8: #{e.message}"
    end

    def blob_inputs(inputs)
      inputs.to_h do |key, value|
        blob = value.is_a?(BlobInput) ? value : BlobInput.new(data: value)
        metadata = blob.metadata.nil? ? "{}" : JSON.generate(blob.metadata)
        [key.to_s, OicanaUniffi::BlobInput.new(data: blob.data.b, metadata: metadata)]
      end
    end

    def pages(pages)
      return nil if pages.nil?

      range = pages.is_a?(Integer) ? (pages..pages) : pages
      raise ArgumentError, "pages must be an Integer or a Range, got #{pages.inspect}" unless range.is_a?(Range)

      first = range.begin
      last = range.end
      last -= 1 if last && range.exclude_end?
      [first, last].compact.each do |index|
        raise InvalidInputError, "page indices must not be negative, got #{pages.inspect}" if index.negative?
      end
      OicanaUniffi::PageRange.new(start: first, _end: last)
    end

    def limits(limits)
      return nil if limits.nil?

      OicanaUniffi::ZipLimits.new(max_entries: limits.max_entries,
                                  max_total_decompressed_bytes: limits.max_total_decompressed_bytes)
    end

    def manifest(json)
      data = JSON.parse(json)
      package = data.fetch("package")
      oicana = data.fetch("oicana")
      Manifest.new(
        package: PackageInfo.new(**package.slice(*PackageInfo.members.map(&:to_s)).transform_keys(&:to_sym)),
        oicana: OicanaConfig.new(
          manifest_version: oicana.fetch("manifestVersion"),
          inputs: oicana.fetch("inputs").map { |input| input_definition(input) },
          validate_json_inputs_by_default: oicana.fetch("validateJsonInputsByDefault"),
          export: ExportConfig.new(pdf: PdfExportConfig.new(standards: oicana.dig("export", "pdf", "standards"),
                                                            tagged: oicana.dig("export", "pdf", "tagged"))),
          fonts: FontConfig.new(require: oicana.dig("fonts", "require"))
        )
      )
    end

    def input_definition(input)
      if input.fetch("type") == "json"
        JsonInputDefinition.new(**input.slice(*JsonInputDefinition.members.map(&:to_s)).transform_keys(&:to_sym))
      else
        BlobInputDefinition.new(key: input.fetch("key"), required: input.fetch("required"),
                                default: blob_fallback(input["default"]),
                                development: blob_fallback(input["development"]))
      end
    end

    def blob_fallback(fallback)
      fallback && BlobFallback.new(file: fallback.fetch("file"), meta: fallback["meta"])
    end
  end
end
