# frozen_string_literal: true

require "zlib"

# Writes uncompressed zip archives for test templates.
module ZipWriter
  def self.stored(files)
    archive = "".b
    central = "".b
    files.each do |name, content|
      name = name.b
      content = content.b
      crc = Zlib.crc32(content)
      offset = archive.bytesize
      archive << [0x04034b50, 10, 0, 0, 0, 0, crc, content.bytesize, content.bytesize, name.bytesize, 0]
                 .pack("VvvvvvVVVvv") << name << content
      central << [0x02014b50, 20, 10, 0, 0, 0, 0, crc, content.bytesize, content.bytesize, name.bytesize,
                  0, 0, 0, 0, 0, offset].pack("VvvvvvvVVVvvvvvVV") << name
    end
    archive << central
    archive << [0x06054b50, 0, 0, files.size, files.size, central.bytesize, archive.bytesize - central.bytesize, 0]
               .pack("VvvvvVVv")
  end
end
