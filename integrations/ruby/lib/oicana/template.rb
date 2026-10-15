# frozen_string_literal: true

module Oicana
  # @api private
  # Frees a generated object on #close, once no call is using it anymore. Objects that
  # are never closed are freed by the garbage collector.
  class NativeHandle
    def initialize(object, free_function, name)
      @object = object
      @free_function = free_function
      @name = name
      @lock = Mutex.new
      @calls = 0
      @closed = false
    end

    def use
      @lock.synchronize do
        raise ClosedError, "#{@name} is already closed" if @closed

        @calls += 1
      end
      begin
        Oicana.translate_errors { yield @object }
      ensure
        @lock.synchronize do
          @calls -= 1
          free if @closed && @calls.zero?
        end
      end
    end

    def close
      @lock.synchronize do
        next if @closed

        @closed = true
        free if @calls.zero?
      end
    end

    private

    def free
      ObjectSpace.undefine_finalizer(@object)
      OicanaUniffi.rust_call(@free_function, @object.instance_variable_get(:@handle))
      @object = nil
    end
  end

  # A packed Oicana template, ready to be compiled repeatedly.
  #
  # Create templates at startup, not per request. A template is safe to use from
  # multiple threads, and native calls release the GVL.
  #
  #   Oicana::Template.open(File.binread("invoice-0.1.0.zip")) do |template|
  #     pdf = template.export_pdf(json_inputs: { "invoice" => { "number" => "2026-001" } })
  #   end
  class Template
    # Diagnostic values for the warnings of the warm-up compilation.
    attr_reader :warnings

    # Load a packed template and compile it once to warm up its caches.
    #
    # json_inputs map keys to JSON documents as Strings, or to values that are encoded
    # with JSON.generate. blob_inputs map keys to BlobInput values or binary Strings.
    def initialize(template, json_inputs: {}, blob_inputs: {}, mode: :development, limits: nil)
      object = Oicana.translate_errors do
        OicanaUniffi::Template.new(template.b, Convert.json_inputs(json_inputs), Convert.blob_inputs(blob_inputs),
                                   Convert.mode(mode), Convert.limits(limits))
      end
      @warnings = Convert.diagnostics(object.warnings)
      @handle = NativeHandle.new(object, :uniffi_oicana_uniffi_fn_free_template, "template")
    end

    # Create a template, yield it, and close it afterwards.
    def self.open(template, **options)
      instance = new(template, **options)
      return instance unless block_given?

      begin
        yield instance
      ensure
        instance.close
      end
    end

    # Compile and export a template in a single native call, without caching anything.
    #
    # Skips the warm-up compilation, so it is the fastest way to render a template
    # exactly once. Returns an ExportOnceResult.
    def self.export_once(template, json_inputs: {}, blob_inputs: {}, format: :pdf, mode: :production,
                         pixels_per_pt: 1.0, pages: nil, limits: nil)
      result = Oicana.translate_errors do
        OicanaUniffi.export_once(template.b, Convert.json_inputs(json_inputs), Convert.blob_inputs(blob_inputs),
                                 Convert.mode(mode), Convert.format(format, pixels_per_pt), Convert.pages(pages),
                                 Convert.limits(limits))
      end
      ExportOnceResult.new(document: result.bytes, warnings: Convert.diagnostics(result.warnings))
    end

    # Compile the template into a CompiledDocument whose pages can be exported repeatedly.
    def compile(json_inputs: {}, blob_inputs: {}, mode: :production)
      json = Convert.json_inputs(json_inputs)
      blobs = Convert.blob_inputs(blob_inputs)
      native_mode = Convert.mode(mode)
      CompiledDocument.new(@handle.use { |template| template.compile(json, blobs, native_mode) })
    end

    # Compile the template and export it as :pdf, :png or :svg.
    #
    # pages is a 0-based page index or Range, for example 0, 1..2 or 3.., and defaults to
    # the whole document.
    def export(format: :pdf, json_inputs: {}, blob_inputs: {}, mode: :production, pixels_per_pt: 1.0, pages: nil)
      document = compile(json_inputs: json_inputs, blob_inputs: blob_inputs, mode: mode)
      begin
        document.export(format: format, pixels_per_pt: pixels_per_pt, pages: pages)
      ensure
        document.close
      end
    end

    # Compile the template and export it to PDF. Tagging is turned off when exporting a
    # subset of pages.
    def export_pdf(json_inputs: {}, blob_inputs: {}, mode: :production, pages: nil)
      export(format: :pdf, json_inputs: json_inputs, blob_inputs: blob_inputs, mode: mode, pages: pages)
    end

    # Compile the template and export it to PNG. Multiple pages are stacked vertically.
    def export_png(json_inputs: {}, blob_inputs: {}, mode: :production, pixels_per_pt: 1.0, pages: nil)
      export(format: :png, json_inputs: json_inputs, blob_inputs: blob_inputs, mode: mode,
             pixels_per_pt: pixels_per_pt, pages: pages)
    end

    # Compile the template and export it to SVG.
    def export_svg(json_inputs: {}, blob_inputs: {}, mode: :production, pages: nil)
      export(format: :svg, json_inputs: json_inputs, blob_inputs: blob_inputs, mode: mode, pages: pages)
    end

    # The template's Manifest.
    def manifest
      Convert.manifest(@handle.use(&:manifest))
    end

    # The content of a text file inside the template.
    def source(path)
      @handle.use { |template| template.source(path) }
    end

    # The content of a file inside the template, as a binary String.
    def file(path)
      @handle.use { |template| template.file(path) }
    end

    # Enable or disable the validation of JSON inputs against their schemas. Validation
    # is enabled by default.
    def validate_inputs=(validate)
      @handle.use { |template| template.set_validate_inputs(validate ? true : false) }
    end

    # Free the template. Documents compiled from it stay usable.
    def close
      @handle.close
    end
  end

  # A compiled document that can be exported repeatedly without compiling again.
  class CompiledDocument
    # PageSize of every page, in document order.
    attr_reader :pages
    # Diagnostic values for the warnings of the compilation.
    attr_reader :warnings

    # @api private
    def initialize(object)
      @pages = object.pages.map { |page| PageSize.new(width: page.width, height: page.height) }.freeze
      @warnings = Convert.diagnostics(object.warnings)
      @handle = NativeHandle.new(object, :uniffi_oicana_uniffi_fn_free_compileddocument, "document")
    end

    # The number of pages.
    def page_count
      @pages.length
    end

    # Export the document as :pdf, :png or :svg. See Template#export for pages.
    def export(format: :pdf, pixels_per_pt: 1.0, pages: nil)
      native_format = Convert.format(format, pixels_per_pt)
      native_pages = Convert.pages(pages)
      @handle.use { |document| document.export(native_format, native_pages) }
    end

    # Export the document to PDF. Tagging is turned off when exporting a subset of pages.
    def export_pdf(pages: nil)
      export(format: :pdf, pages: pages)
    end

    # Export the document to PNG. Multiple pages are stacked vertically.
    def export_png(pixels_per_pt: 1.0, pages: nil)
      export(format: :png, pixels_per_pt: pixels_per_pt, pages: pages)
    end

    # Export the document to SVG.
    def export_svg(pages: nil)
      export(format: :svg, pages: pages)
    end

    # Free the document.
    def close
      @handle.close
    end
  end
end
