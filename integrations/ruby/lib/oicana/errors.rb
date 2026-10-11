# frozen_string_literal: true

module Oicana
  # Base class of all errors raised by Oicana. The message includes rendered
  # compilation diagnostics.
  class Error < StandardError; end

  # The packed template could not be read or its manifest is invalid.
  class InvalidTemplateError < Error; end

  # An input value, its blob metadata, or a page range is invalid.
  class InvalidInputError < Error; end

  # The template failed to compile.
  class CompilationError < Error; end

  # The requested file is not part of the template.
  class FileNotFoundError < Error; end

  # The compiled document could not be exported.
  class ExportError < Error; end

  # A template or document is no longer registered.
  class NotFoundError < Error; end

  # An unexpected failure inside Oicana.
  class InternalError < Error; end

  # A template or document was used after #close.
  class ClosedError < Error; end

  ERRORS = {
    "InvalidTemplate" => InvalidTemplateError,
    "InvalidInput" => InvalidInputError,
    "Compilation" => CompilationError,
    "FileNotFound" => FileNotFoundError,
    "Export" => ExportError,
    "NotFound" => NotFoundError,
    "Internal" => InternalError
  }.freeze
  private_constant :ERRORS

  # @api private
  # Runs the block and translates errors of the generated bindings.
  def self.translate_errors
    yield
  rescue OicanaUniffi::OicanaError::InvalidTemplate, OicanaUniffi::OicanaError::InvalidInput,
         OicanaUniffi::OicanaError::Compilation, OicanaUniffi::OicanaError::FileNotFound,
         OicanaUniffi::OicanaError::Export, OicanaUniffi::OicanaError::NotFound,
         OicanaUniffi::OicanaError::Internal => e
    raise ERRORS.fetch(e.class.name.split("::").last), e.message
  rescue OicanaUniffi::InternalError => e
    raise InternalError, e.message
  end
end
