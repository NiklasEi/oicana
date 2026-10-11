# frozen_string_literal: true

require_relative "oicana/version"
require_relative "oicana/native"
require_relative "oicana/oicana_uniffi"
require_relative "oicana/errors"
require_relative "oicana/types"
require_relative "oicana/template"

Oicana::Native.patch_bindings

# Generate documents from Typst templates packed with the Oicana CLI.
module Oicana
  # Make fonts available to every template created afterwards. Takes the content of font
  # files; data that holds no font is ignored. Returns the number of font faces added.
  def self.register_fonts(*fonts)
    OicanaUniffi.register_fonts(fonts.flatten.map(&:b))
  end

  # Make font files, or every font file in the given directories, available to every
  # template created afterwards. Returns the number of font faces added.
  def self.register_font_paths(*paths)
    OicanaUniffi.register_font_paths(paths.flatten.map(&:to_s))
  end

  # The RegisteredFont faces registered with register_fonts or register_font_paths.
  def self.registered_fonts
    OicanaUniffi.registered_fonts.map { |font| RegisteredFont.new(family: font.family, path: font.path) }
  end

  # Drop all registered fonts. Existing templates keep the fonts they were created with.
  def self.clear_fonts
    OicanaUniffi.clear_fonts
  end

  # Set the coloring of compilation diagnostics for all templates: :none or :ansi.
  def self.configure_diagnostic_color(color)
    native = { none: OicanaUniffi::DiagnosticColor::NONE, ansi: OicanaUniffi::DiagnosticColor::ANSI }.fetch(color) do
      raise ArgumentError, "unknown diagnostic color #{color.inspect}, use :none or :ansi"
    end
    OicanaUniffi.configure_diagnostic_color(native)
  end

  # Evict cache entries after each compilation that were not used within the last
  # max_age evictions. nil disables the eviction, 0 clears the whole cache every time.
  # The default is 10.
  def self.configure_automatic_cache_eviction(max_age)
    OicanaUniffi.configure_automatic_cache_eviction(max_age)
  end

  # Evict cache entries that were not used within the last max_age evictions.
  def self.evict_cache(max_age)
    OicanaUniffi.evict_cache(max_age)
  end

  # The version of the loaded native library.
  def self.native_version
    OicanaUniffi.version
  end
end
