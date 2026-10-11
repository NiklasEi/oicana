# frozen_string_literal: true

require "ffi"

module Oicana
  # Loads the native library and adjusts the generated bindings in lib/oicana/oicana_uniffi.rb.
  module Native
    # Makes every native call release the GVL, so other Ruby threads keep running while
    # a template compiles or exports.
    module BlockingCalls
      def attach_function(*args)
        options = args.last.is_a?(Hash) ? args.pop : {}
        super(*args, options.merge(blocking: true))
      end
    end

    # Copies strings into native buffers at once. The generated code goes through
    # String#bytes, which allocates an Integer for every byte.
    module BulkWrites
      def write(value)
        reserve(value.bytesize) do
          @rust_buf.data.put_bytes(@rust_buf.len, value)
        end
      end

      def write_String(value)
        value = OicanaUniffi.uniffi_utf8(value)
        pack_into 4, "l>", value.bytesize
        write value
      end

      def write_Bytes(value)
        value = OicanaUniffi.uniffi_bytes(value)
        pack_into 4, "l>", value.bytesize
        write value
      end
    end

    # The native library, either from OICANA_LIBRARY_PATH or shipped in the platform gem.
    def self.library_path
      ENV.fetch("OICANA_LIBRARY_PATH") do
        path = File.expand_path("native/#{FFI.map_library_name("oicana_uniffi")}", __dir__)
        return path if File.exist?(path)

        raise LoadError, missing_library_message
      end
    end

    def self.missing_library_message
      platform = Gem::Platform.local
      if %w[x86_64 x64 aarch64 arm64].include?(platform.cpu) && %w[linux darwin mingw].include?(platform.os)
        "The oicana gem without a native library is installed, but #{platform} has a platform gem that " \
          "contains it. Check that Bundler's force_ruby_platform is not set, and add the platform to your " \
          "Gemfile.lock with `bundle lock --add-platform #{platform}`."
      else
        "Oicana has no native library for #{platform}. Supported are Linux (glibc and musl), macOS and " \
          "Windows, each on x64 and arm64. Set OICANA_LIBRARY_PATH to use a library built from source."
      end
    end

    # Called by the generated bindings, see uniffi.toml.
    def self.prepare(library)
      library.singleton_class.prepend(BlockingCalls)
      library_path
    end

    def self.patch_bindings
      OicanaUniffi.const_get(:RustBufferBuilder).prepend(BulkWrites)
    end
  end
end
