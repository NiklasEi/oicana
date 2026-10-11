# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "zip_writer"
require "oicana"

class OicanaTest < Minitest::Test
  ROOT = File.expand_path("../../..", __dir__)
  OUTPUT = File.expand_path("testOutput", __dir__)
  TEST_FAMILY = "Oicana Test"

  def asset(*path)
    File.binread(File.join(ROOT, "assets", *path))
  end

  def e2e_template
    File.binread(File.join(ROOT, "e2e-tests", "template", "oicana-e2e-test-x.y.z.zip"))
  end

  def minimal_template(main, required_fonts: [])
    manifest = +"[package]\nname = \"test\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\n\n" \
                "[tool.oicana]\nmanifest_version = 1\n"
    manifest << "\n[tool.oicana.fonts]\nrequire = #{required_fonts.inspect}\n" unless required_fonts.empty?
    ZipWriter.stored("typst.toml" => manifest, "main.typ" => main)
  end

  def write_output(name, data)
    FileUtils.mkdir_p(OUTPUT)
    File.binwrite(File.join(OUTPUT, name), data)
  end

  def blob(foo)
    Oicana::BlobInput.new(data: asset("inputs", "input.txt"),
                          metadata: { image_format: "jpeg", foo: foo, bar: %w[input two] })
  end

  def test_development
    Oicana::Template.open(e2e_template) do |template|
      write_output("development.png", template.export_png(mode: :development))
    end
  end

  def test_production
    Oicana::Template.open(e2e_template) do |template|
      image = template.export_png(json_inputs: { "development-json" => asset("inputs", "input.json") },
                                  blob_inputs: { "development-blob" => blob(43) })
      write_output("production.png", image)
    end
  end

  def test_all_inputs
    json = asset("inputs", "input.json")
    Oicana::Template.open(e2e_template) do |template|
      image = template.export_png(
        json_inputs: { "default-json" => json, "development-json" => json, "both-json" => json },
        blob_inputs: { "default-blob" => blob(42), "development-blob" => blob(43), "both-blob" => blob(44) }
      )
      write_output("all-inputs.png", image)
    end
  end

  def test_export_formats
    Oicana::Template.open(e2e_template) do |template|
      assert template.export_pdf(mode: :development).start_with?("%PDF")
      assert template.export_png(mode: :development, pixels_per_pt: 2).start_with?("\x89PNG".b)
      assert_includes template.export_svg(mode: :development), "<svg"
      assert template.export(format: :png, mode: :development).start_with?("\x89PNG".b)
    end
  end

  def test_export_defaults_to_production_mode
    Oicana::Template.open(e2e_template) do |template|
      error = assert_raises(Oicana::CompilationError) { template.export_pdf }
      assert_includes error.message, "No value for the required input"
    end
  end

  def test_production_warm_up_needs_inputs
    assert_raises(Oicana::CompilationError) { Oicana::Template.new(e2e_template, mode: :production) }
  end

  def test_manifest
    manifest = Oicana::Template.open(e2e_template, &:manifest)

    assert_equal "oicana-e2e-test", manifest.package.name
    assert_equal "0.1.0", manifest.package.version
    assert_equal 1, manifest.oicana.manifest_version
    assert manifest.oicana.validate_json_inputs_by_default
    assert_empty manifest.oicana.export.pdf.standards
    assert_empty manifest.oicana.fonts.require

    inputs = manifest.oicana.inputs.to_h { |input| [input.key, input] }
    json_input = inputs.fetch("development-json")
    assert_instance_of Oicana::JsonInputDefinition, json_input
    assert_equal "input.schema.json", json_input.schema
    assert_equal "development.json", json_input.development
    assert_nil json_input.default
    assert json_input.validate

    blob_input = inputs.fetch("default-blob")
    assert_instance_of Oicana::BlobInputDefinition, blob_input
    assert_equal "default.txt", blob_input.default.file
    assert_equal "png", blob_input.default.meta["image_format"]
    assert_nil blob_input.development
  end

  def test_template_files
    Oicana::Template.open(e2e_template) do |template|
      refute_empty template.source("main.typ")
      assert_equal Encoding::BINARY, template.file("default.txt").encoding
      assert_raises(Oicana::FileNotFoundError) { template.file("missing.txt") }
    end
  end

  def test_validate_inputs
    Oicana::Template.open(e2e_template) do |template|
      inputs = { "development-json" => { unexpected: true } }
      assert_raises(Oicana::InvalidInputError) { template.export_pdf(json_inputs: inputs, mode: :development) }

      template.validate_inputs = false
      begin
        template.export_pdf(json_inputs: inputs, mode: :development)
      rescue Oicana::CompilationError
        # The template may still fail on the unexpected value, but not on its schema.
      end
    end
  end

  def test_binary_json_input_with_non_ascii
    json = JSON.generate({ unexpected: "ä" }).b
    Oicana::Template.open(e2e_template) do |template|
      error = assert_raises(Oicana::InvalidInputError) do
        template.export_pdf(json_inputs: { "development-json" => json }, mode: :development)
      end
      refute_match(/UTF-8/, error.message)
      assert_raises(Oicana::InvalidInputError) do
        template.export_pdf(json_inputs: { "development-json" => "\xff".b }, mode: :development)
      end
    end
  end

  def test_forked_child_fails_instead_of_hanging
    skip "fork is not available" unless Process.respond_to?(:fork)

    template = Oicana::Template.new(e2e_template)
    pid = fork do
      Oicana::Template.new(e2e_template)
      exit!(1)
    rescue Oicana::Error => e
      exit!(e.message.include?("forked process") ? 0 : 2)
    end
    deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 30
    status = nil
    until (status = Process.wait2(pid, Process::WNOHANG)&.last)
      if Process.clock_gettime(Process::CLOCK_MONOTONIC) > deadline
        Process.kill(:KILL, pid)
        flunk "the forked child hung"
      end
      sleep 0.05
    end
    assert_equal 0, status.exitstatus
    template.close
  end

  def test_compiled_document_outlives_template
    template = Oicana::Template.new(e2e_template)
    document = template.compile(mode: :development)
    template.close

    refute_empty document.pages
    assert_equal document.pages.length, document.page_count
    assert_operator document.pages.first.width, :>, 0
    assert document.export_pdf(pages: 0).start_with?("%PDF")
    assert document.export_png(pages: ..0).start_with?("\x89PNG".b)
    assert_includes document.export_svg(pages: 0...1), "<svg"
    assert_raises(Oicana::ExportError) { document.export_svg(pages: document.page_count..) }
    assert_raises(Oicana::InvalidInputError) { document.export_pdf(pages: -1) }
  ensure
    document&.close
  end

  def test_closed
    template = Oicana::Template.new(e2e_template)
    document = template.compile(mode: :development)
    template.close
    template.close
    document.close

    assert_raises(Oicana::ClosedError) { template.compile(mode: :development) }
    assert_raises(Oicana::ClosedError) { document.export_pdf }
  end

  def test_invalid_template
    assert_raises(Oicana::InvalidTemplateError) { Oicana::Template.new("not a zip") }
  end

  def test_zip_limits
    assert_raises(Oicana::InvalidTemplateError) do
      Oicana::Template.new(e2e_template, limits: Oicana::ZipLimits.new(max_entries: 1))
    end
  end

  def test_invalid_arguments
    Oicana::Template.open(e2e_template) do |template|
      assert_raises(ArgumentError) { template.export(format: :docx, mode: :development) }
      assert_raises(ArgumentError) { template.export_pdf(mode: :draft) }
    end
  end

  def test_export_once
    result = Oicana::Template.export_once(e2e_template, mode: :development)
    assert result.document.start_with?("%PDF")

    result = Oicana::Template.export_once(minimal_template("#set text(font: \"No Such Font\")\nHello"),
                                          format: :svg, pages: 0)
    assert_includes result.warnings, "No Such Font"

    assert_raises(Oicana::CompilationError) do
      Oicana::Template.export_once(minimal_template("#panic(\"broken\")"))
    end
  end

  def test_fonts
    Oicana.clear_fonts
    template = minimal_template("#set text(font: \"#{TEST_FAMILY}\")\nHello", required_fonts: [TEST_FAMILY])
    assert_raises(Oicana::InvalidTemplateError) { Oicana::Template.new(template) }

    assert_equal 1, Oicana.register_fonts(asset("fonts", "oicana-test-font.ttf"), "no font")
    assert_equal [Oicana::RegisteredFont.new(family: TEST_FAMILY, path: nil)], Oicana.registered_fonts
    Oicana::Template.new(template).close

    Oicana.clear_fonts
    assert_equal 1, Oicana.register_font_paths(File.join(ROOT, "assets", "fonts"))
    assert Oicana.registered_fonts.first.path.end_with?("oicana-test-font.ttf")
  ensure
    Oicana.clear_fonts
  end

  def test_diagnostic_color
    broken = minimal_template("#panic(\"broken\")")
    Oicana.configure_diagnostic_color(:ansi)
    error = assert_raises(Oicana::CompilationError) { Oicana::Template.export_once(broken) }
    assert_includes error.message, "\e["

    Oicana.configure_diagnostic_color(:none)
    error = assert_raises(Oicana::CompilationError) { Oicana::Template.export_once(broken) }
    refute_includes error.message, "\e["
  ensure
    Oicana.configure_diagnostic_color(:none)
  end

  def test_cache_eviction
    Oicana::Template.open(e2e_template) do |template|
      Oicana.configure_automatic_cache_eviction(nil)
      Oicana.evict_cache(0)
      Oicana.configure_automatic_cache_eviction(0)
      assert template.export_pdf(mode: :development).start_with?("%PDF")
    end
  ensure
    Oicana.configure_automatic_cache_eviction(10)
  end

  def test_native_version
    assert_equal Oicana::VERSION, Oicana.native_version
  end

  def test_bulk_writes_are_patched_in
    assert_includes OicanaUniffi.const_get(:RustBufferBuilder).ancestors, Oicana::Native::BulkWrites
  end

  def test_concurrent_exports
    Oicana::Template.open(e2e_template) do |template|
      threads = Array.new(8) { Thread.new { template.export_pdf(mode: :development) } }
      threads.each { |thread| assert thread.value.start_with?("%PDF") }
    end
  end

  def test_native_calls_release_the_gvl
    Oicana::Template.open(e2e_template) do |template|
      document = template.compile(mode: :development)
      window = {}
      thread = Thread.new do
        window[:start] = Process.clock_gettime(Process::CLOCK_MONOTONIC)
        # A single native call, made slow through a high resolution.
        document.export_png(pixels_per_pt: 8)
        window[:end] = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      end
      ticks = []
      while thread.alive?
        ticks << Process.clock_gettime(Process::CLOCK_MONOTONIC)
        sleep 0.001
      end
      thread.join
      document.close

      margin = (window[:end] - window[:start]) / 4
      inside = ticks.select { |tick| tick > window[:start] + margin && tick < window[:end] - margin }
      refute_empty inside, "the main thread never ran while the export was in flight"
    end
  end
end
