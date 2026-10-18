package oicana_uniffi

// #include <oicana_uniffi.h>
import "C"

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"runtime"
	"sync/atomic"
	"unsafe"
)

// This is needed, because as of go 1.24
// type RustBuffer C.RustBuffer cannot have methods,
// RustBuffer is treated as non-local type
type GoRustBuffer struct {
	inner C.RustBuffer
}

type RustBufferI interface {
	AsReader() *bytes.Reader
	Free()
	ToGoBytes() []byte
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

// C.RustBuffer fields exposed as an interface so they can be accessed in different Go packages.
// See https://github.com/golang/go/issues/13467
type ExternalCRustBuffer interface {
	Data() unsafe.Pointer
	Len() uint64
	Capacity() uint64
}

func RustBufferFromC(b C.RustBuffer) ExternalCRustBuffer {
	return GoRustBuffer{
		inner: b,
	}
}

func CFromRustBuffer(b ExternalCRustBuffer) C.RustBuffer {
	return C.RustBuffer{
		capacity: C.uint64_t(b.Capacity()),
		len:      C.uint64_t(b.Len()),
		data:     (*C.uchar)(b.Data()),
	}
}

func RustBufferFromExternal(b ExternalCRustBuffer) GoRustBuffer {
	return GoRustBuffer{
		inner: C.RustBuffer{
			capacity: C.uint64_t(b.Capacity()),
			len:      C.uint64_t(b.Len()),
			data:     (*C.uchar)(b.Data()),
		},
	}
}

func (cb GoRustBuffer) Capacity() uint64 {
	return uint64(cb.inner.capacity)
}

func (cb GoRustBuffer) Len() uint64 {
	return uint64(cb.inner.len)
}

func (cb GoRustBuffer) Data() unsafe.Pointer {
	return unsafe.Pointer(cb.inner.data)
}

func (cb GoRustBuffer) AsReader() *bytes.Reader {
	b := unsafe.Slice((*byte)(cb.inner.data), C.uint64_t(cb.inner.len))
	return bytes.NewReader(b)
}

func (cb GoRustBuffer) Free() {
	rustCall(func(status *C.RustCallStatus) bool {
		C.ffi_oicana_uniffi_rustbuffer_free(cb.inner, status)
		return false
	})
}

func (cb GoRustBuffer) ToGoBytes() []byte {
	return C.GoBytes(unsafe.Pointer(cb.inner.data), C.int(cb.inner.len))
}

func stringToRustBuffer(str string) C.RustBuffer {
	return bytesToRustBuffer([]byte(str))
}

func bytesToRustBuffer(b []byte) C.RustBuffer {
	if len(b) == 0 {
		return C.RustBuffer{}
	}
	// We can pass the pointer along here, as it is pinned
	// for the duration of this call
	foreign := C.ForeignBytes{
		len:  C.int(len(b)),
		data: (*C.uchar)(unsafe.Pointer(&b[0])),
	}

	return rustCall(func(status *C.RustCallStatus) C.RustBuffer {
		return C.ffi_oicana_uniffi_rustbuffer_from_bytes(foreign, status)
	})
}

type BufLifter[GoType any] interface {
	Lift(value RustBufferI) GoType
}

type BufLowerer[GoType any] interface {
	Lower(value GoType) C.RustBuffer
}

type BufReader[GoType any] interface {
	Read(reader io.Reader) GoType
}

type BufWriter[GoType any] interface {
	Write(writer io.Writer, value GoType)
}

func LowerIntoRustBuffer[GoType any](bufWriter BufWriter[GoType], value GoType) C.RustBuffer {
	// This might be not the most efficient way but it does not require knowing allocation size
	// beforehand
	var buffer bytes.Buffer
	bufWriter.Write(&buffer, value)

	bytes, err := io.ReadAll(&buffer)
	if err != nil {
		panic(fmt.Errorf("reading written data: %w", err))
	}
	return bytesToRustBuffer(bytes)
}

func LiftFromRustBuffer[GoType any](bufReader BufReader[GoType], rbuf RustBufferI) GoType {
	defer rbuf.Free()
	reader := rbuf.AsReader()
	item := bufReader.Read(reader)
	if reader.Len() > 0 {
		// TODO: Remove this
		leftover, _ := io.ReadAll(reader)
		panic(fmt.Errorf("Junk remaining in buffer after lifting: %s", string(leftover)))
	}
	return item
}

func rustCallWithError[E any, U any](converter BufReader[E], callback func(*C.RustCallStatus) U) (U, E) {
	var status C.RustCallStatus
	returnValue := callback(&status)
	err := checkCallStatus(converter, status)
	return returnValue, err
}

func checkCallStatus[E any](converter BufReader[E], status C.RustCallStatus) E {
	switch status.code {
	case 0:
		var zero E
		return zero
	case 1:
		return LiftFromRustBuffer(converter, GoRustBuffer{inner: status.errorBuf})
	case 2:
		// when the rust code sees a panic, it tries to construct a rustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{inner: status.errorBuf})))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		panic(fmt.Errorf("unknown status code: %d", status.code))
	}
}

func checkCallStatusUnknown(status C.RustCallStatus) error {
	switch status.code {
	case 0:
		return nil
	case 1:
		panic(fmt.Errorf("function not returning an error returned an error"))
	case 2:
		// when the rust code sees a panic, it tries to construct a C.RustBuffer
		// with the message.  but if that code panics, then it just sends back
		// an empty buffer.
		if status.errorBuf.len > 0 {
			panic(fmt.Errorf("%s", FfiConverterStringINSTANCE.Lift(GoRustBuffer{
				inner: status.errorBuf,
			})))
		} else {
			panic(fmt.Errorf("Rust panicked while handling Rust panic"))
		}
	default:
		return fmt.Errorf("unknown status code: %d", status.code)
	}
}

func rustCall[U any](callback func(*C.RustCallStatus) U) U {
	returnValue, err := rustCallWithError[error](nil, callback)
	if err != nil {
		panic(err)
	}
	return returnValue
}

type NativeError interface {
	AsError() error
}

func writeInt8(writer io.Writer, value int8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint8(writer io.Writer, value uint8) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt16(writer io.Writer, value int16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint16(writer io.Writer, value uint16) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt32(writer io.Writer, value int32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint32(writer io.Writer, value uint32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeInt64(writer io.Writer, value int64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeUint64(writer io.Writer, value uint64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat32(writer io.Writer, value float32) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func writeFloat64(writer io.Writer, value float64) {
	if err := binary.Write(writer, binary.BigEndian, value); err != nil {
		panic(err)
	}
}

func readInt8(reader io.Reader) int8 {
	var result int8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint8(reader io.Reader) uint8 {
	var result uint8
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt16(reader io.Reader) int16 {
	var result int16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint16(reader io.Reader) uint16 {
	var result uint16
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt32(reader io.Reader) int32 {
	var result int32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint32(reader io.Reader) uint32 {
	var result uint32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readInt64(reader io.Reader) int64 {
	var result int64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readUint64(reader io.Reader) uint64 {
	var result uint64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat32(reader io.Reader) float32 {
	var result float32
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func readFloat64(reader io.Reader) float64 {
	var result float64
	if err := binary.Read(reader, binary.BigEndian, &result); err != nil {
		panic(err)
	}
	return result
}

func init() {

	uniffiCheckChecksums()
}

func uniffiCheckChecksums() {
	// Get the bindings contract version from our ComponentInterface
	bindingsContractVersion := 30
	// Get the scaffolding contract version by calling the into the dylib
	scaffoldingContractVersion := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint32_t {
		return C.ffi_oicana_uniffi_uniffi_contract_version()
	})
	if bindingsContractVersion != int(scaffoldingContractVersion) {
		// If this happens try cleaning and rebuilding your project
		panic("oicana_uniffi: UniFFI contract version mismatch")
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_clear_fonts()
		})
		if checksum != 18964 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_clear_fonts: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_configure_automatic_cache_eviction()
		})
		if checksum != 37373 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_configure_automatic_cache_eviction: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_configure_diagnostic_color()
		})
		if checksum != 47395 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_configure_diagnostic_color: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_evict_cache()
		})
		if checksum != 29494 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_evict_cache: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_export_once()
		})
		if checksum != 59019 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_export_once: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_register_font_paths()
		})
		if checksum != 61184 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_register_font_paths: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_register_fonts()
		})
		if checksum != 42024 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_register_fonts: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_registered_fonts()
		})
		if checksum != 17013 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_registered_fonts: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_func_version()
		})
		if checksum != 58374 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_func_version: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_compileddocument_export()
		})
		if checksum != 55777 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_compileddocument_export: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_compileddocument_pages()
		})
		if checksum != 1025 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_compileddocument_pages: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_compileddocument_warnings()
		})
		if checksum != 4493 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_compileddocument_warnings: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_compile()
		})
		if checksum != 35206 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_compile: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_file()
		})
		if checksum != 23146 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_file: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_manifest()
		})
		if checksum != 52272 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_manifest: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_set_validate_inputs()
		})
		if checksum != 6499 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_set_validate_inputs: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_source()
		})
		if checksum != 24155 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_source: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_method_template_warnings()
		})
		if checksum != 11791 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_method_template_warnings: UniFFI API checksum mismatch")
		}
	}
	{
		checksum := rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint16_t {
			return C.uniffi_oicana_uniffi_checksum_constructor_template_new()
		})
		if checksum != 60852 {
			// If this happens try cleaning and rebuilding your project
			panic("oicana_uniffi: uniffi_oicana_uniffi_checksum_constructor_template_new: UniFFI API checksum mismatch")
		}
	}
}

type FfiConverterUint32 struct{}

var FfiConverterUint32INSTANCE = FfiConverterUint32{}

func (FfiConverterUint32) Lower(value uint32) C.uint32_t {
	return C.uint32_t(value)
}

func (FfiConverterUint32) Write(writer io.Writer, value uint32) {
	writeUint32(writer, value)
}

func (FfiConverterUint32) Lift(value C.uint32_t) uint32 {
	return uint32(value)
}

func (FfiConverterUint32) Read(reader io.Reader) uint32 {
	return readUint32(reader)
}

type FfiDestroyerUint32 struct{}

func (FfiDestroyerUint32) Destroy(_ uint32) {}

type FfiConverterUint64 struct{}

var FfiConverterUint64INSTANCE = FfiConverterUint64{}

func (FfiConverterUint64) Lower(value uint64) C.uint64_t {
	return C.uint64_t(value)
}

func (FfiConverterUint64) Write(writer io.Writer, value uint64) {
	writeUint64(writer, value)
}

func (FfiConverterUint64) Lift(value C.uint64_t) uint64 {
	return uint64(value)
}

func (FfiConverterUint64) Read(reader io.Reader) uint64 {
	return readUint64(reader)
}

type FfiDestroyerUint64 struct{}

func (FfiDestroyerUint64) Destroy(_ uint64) {}

type FfiConverterFloat32 struct{}

var FfiConverterFloat32INSTANCE = FfiConverterFloat32{}

func (FfiConverterFloat32) Lower(value float32) C.float {
	return C.float(value)
}

func (FfiConverterFloat32) Write(writer io.Writer, value float32) {
	writeFloat32(writer, value)
}

func (FfiConverterFloat32) Lift(value C.float) float32 {
	return float32(value)
}

func (FfiConverterFloat32) Read(reader io.Reader) float32 {
	return readFloat32(reader)
}

type FfiDestroyerFloat32 struct{}

func (FfiDestroyerFloat32) Destroy(_ float32) {}

type FfiConverterFloat64 struct{}

var FfiConverterFloat64INSTANCE = FfiConverterFloat64{}

func (FfiConverterFloat64) Lower(value float64) C.double {
	return C.double(value)
}

func (FfiConverterFloat64) Write(writer io.Writer, value float64) {
	writeFloat64(writer, value)
}

func (FfiConverterFloat64) Lift(value C.double) float64 {
	return float64(value)
}

func (FfiConverterFloat64) Read(reader io.Reader) float64 {
	return readFloat64(reader)
}

type FfiDestroyerFloat64 struct{}

func (FfiDestroyerFloat64) Destroy(_ float64) {}

type FfiConverterBool struct{}

var FfiConverterBoolINSTANCE = FfiConverterBool{}

func (FfiConverterBool) Lower(value bool) C.int8_t {
	if value {
		return C.int8_t(1)
	}
	return C.int8_t(0)
}

func (FfiConverterBool) Write(writer io.Writer, value bool) {
	if value {
		writeInt8(writer, 1)
	} else {
		writeInt8(writer, 0)
	}
}

func (FfiConverterBool) Lift(value C.int8_t) bool {
	return value != 0
}

func (FfiConverterBool) Read(reader io.Reader) bool {
	return readInt8(reader) != 0
}

type FfiDestroyerBool struct{}

func (FfiDestroyerBool) Destroy(_ bool) {}

type FfiConverterString struct{}

var FfiConverterStringINSTANCE = FfiConverterString{}

func (FfiConverterString) Lift(rb RustBufferI) string {
	defer rb.Free()
	reader := rb.AsReader()
	b, err := io.ReadAll(reader)
	if err != nil {
		panic(fmt.Errorf("reading reader: %w", err))
	}
	return string(b)
}

func (FfiConverterString) Read(reader io.Reader) string {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil && err != io.EOF {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading string, expected %d, read %d", length, read_length))
	}
	return string(buffer)
}

func (FfiConverterString) Lower(value string) C.RustBuffer {
	return stringToRustBuffer(value)
}

func (c FfiConverterString) LowerExternal(value string) ExternalCRustBuffer {
	return RustBufferFromC(stringToRustBuffer(value))
}

func (FfiConverterString) Write(writer io.Writer, value string) {
	if len(value) > math.MaxInt32 {
		panic("String is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := io.WriteString(writer, value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing string, expected %d, written %d", len(value), write_length))
	}
}

type FfiDestroyerString struct{}

func (FfiDestroyerString) Destroy(_ string) {}

type FfiConverterBytes struct{}

var FfiConverterBytesINSTANCE = FfiConverterBytes{}

func (c FfiConverterBytes) Lower(value []byte) C.RustBuffer {
	return LowerIntoRustBuffer[[]byte](c, value)
}

func (c FfiConverterBytes) LowerExternal(value []byte) ExternalCRustBuffer {
	return RustBufferFromC(c.Lower(value))
}

func (c FfiConverterBytes) Write(writer io.Writer, value []byte) {
	if len(value) > math.MaxInt32 {
		panic("[]byte is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	write_length, err := writer.Write(value)
	if err != nil {
		panic(err)
	}
	if write_length != len(value) {
		panic(fmt.Errorf("bad write length when writing []byte, expected %d, written %d", len(value), write_length))
	}
}

func (c FfiConverterBytes) Lift(rb RustBufferI) []byte {
	return LiftFromRustBuffer[[]byte](c, rb)
}

func (c FfiConverterBytes) Read(reader io.Reader) []byte {
	length := readInt32(reader)
	buffer := make([]byte, length)
	read_length, err := reader.Read(buffer)
	if err != nil && err != io.EOF {
		panic(err)
	}
	if read_length != int(length) {
		panic(fmt.Errorf("bad read length when reading []byte, expected %d, read %d", length, read_length))
	}
	return buffer
}

type FfiDestroyerBytes struct{}

func (FfiDestroyerBytes) Destroy(_ []byte) {}

// Below is an implementation of synchronization requirements outlined in the link.
// https://github.com/mozilla/uniffi-rs/blob/0dc031132d9493ca812c3af6e7dd60ad2ea95bf0/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt#L31

type FfiObject struct {
	handle        C.uint64_t
	callCounter   atomic.Int64
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t
	freeFunction  func(C.uint64_t, *C.RustCallStatus)
	destroyed     atomic.Bool
}

func newFfiObject(
	handle C.uint64_t,
	cloneFunction func(C.uint64_t, *C.RustCallStatus) C.uint64_t,
	freeFunction func(C.uint64_t, *C.RustCallStatus),
) FfiObject {
	return FfiObject{
		handle:        handle,
		cloneFunction: cloneFunction,
		freeFunction:  freeFunction,
	}
}

func (ffiObject *FfiObject) incrementPointer(debugName string) C.uint64_t {
	for {
		counter := ffiObject.callCounter.Load()
		if counter <= -1 {
			panic(fmt.Errorf("%v object has already been destroyed", debugName))
		}
		if counter == math.MaxInt64 {
			panic(fmt.Errorf("%v object call counter would overflow", debugName))
		}
		if ffiObject.callCounter.CompareAndSwap(counter, counter+1) {
			break
		}
	}

	return rustCall(func(status *C.RustCallStatus) C.uint64_t {
		return ffiObject.cloneFunction(ffiObject.handle, status)
	})
}

func (ffiObject *FfiObject) decrementPointer() {
	if ffiObject.callCounter.Add(-1) == -1 {
		ffiObject.freeRustArcPtr()
	}
}

func (ffiObject *FfiObject) destroy() {
	if ffiObject.destroyed.CompareAndSwap(false, true) {
		if ffiObject.callCounter.Add(-1) == -1 {
			ffiObject.freeRustArcPtr()
		}
	}
}

func (ffiObject *FfiObject) freeRustArcPtr() {
	if ffiObject.handle == 0 {
		return
	}
	rustCall(func(status *C.RustCallStatus) int32 {
		ffiObject.freeFunction(ffiObject.handle, status)
		return 0
	})
}

// A compiled document that can be exported repeatedly without compiling again.
//
// Dropping it frees the document.
type CompiledDocumentInterface interface {
	// Export the document, optionally restricted to a range of pages.
	Export(format ExportFormat, pages *PageRange) ([]byte, error)
	// Sizes of every page, in document order.
	Pages() []PageSize
	// Warnings of the compilation.
	Warnings() []Diagnostic
}

// A compiled document that can be exported repeatedly without compiling again.
//
// Dropping it frees the document.
type CompiledDocument struct {
	ffiObject FfiObject
}

// Export the document, optionally restricted to a range of pages.
func (_self *CompiledDocument) Export(format ExportFormat, pages *PageRange) ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*CompiledDocument")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_compileddocument_export(
				_pointer, FfiConverterExportFormatINSTANCE.Lower(format), FfiConverterOptionalPageRangeINSTANCE.Lower(pages), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []byte
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBytesINSTANCE.Lift(_uniffiRV), nil
	}
}

// Sizes of every page, in document order.
func (_self *CompiledDocument) Pages() []PageSize {
	_pointer := _self.ffiObject.incrementPointer("*CompiledDocument")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequencePageSizeINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_compileddocument_pages(
				_pointer, _uniffiStatus),
		}
	}))
}

// Warnings of the compilation.
func (_self *CompiledDocument) Warnings() []Diagnostic {
	_pointer := _self.ffiObject.incrementPointer("*CompiledDocument")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequenceDiagnosticINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_compileddocument_warnings(
				_pointer, _uniffiStatus),
		}
	}))
}
func (object *CompiledDocument) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterCompiledDocument struct{}

var FfiConverterCompiledDocumentINSTANCE = FfiConverterCompiledDocument{}

func (c FfiConverterCompiledDocument) Lift(handle C.uint64_t) *CompiledDocument {
	result := &CompiledDocument{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_oicana_uniffi_fn_clone_compileddocument(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_oicana_uniffi_fn_free_compileddocument(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*CompiledDocument).Destroy)
	return result
}

func (c FfiConverterCompiledDocument) Read(reader io.Reader) *CompiledDocument {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterCompiledDocument) Lower(value *CompiledDocument) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*CompiledDocument")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterCompiledDocument) Write(writer io.Writer, value *CompiledDocument) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalCompiledDocument(handle uint64) *CompiledDocument {
	return FfiConverterCompiledDocumentINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalCompiledDocument(value *CompiledDocument) uint64 {
	return uint64(FfiConverterCompiledDocumentINSTANCE.Lower(value))
}

type FfiDestroyerCompiledDocument struct{}

func (_ FfiDestroyerCompiledDocument) Destroy(value *CompiledDocument) {
	value.Destroy()
}

// A registered template, ready for repeated compilation.
//
// Dropping it removes the template from the cache. Documents compiled from it stay
// usable.
type TemplateInterface interface {
	// Compile the template with the given inputs.
	Compile(jsonInputs map[string]string, blobInputs map[string]BlobInput, mode CompilationMode) (*CompiledDocument, error)
	// Raw bytes of a file inside the template.
	File(path string) ([]byte, error)
	// The template manifest as JSON.
	Manifest() (string, error)
	// Enable or disable validating JSON inputs against their schemas. Enabled by default.
	SetValidateInputs(validate bool) error
	// Source text of a file inside the template.
	Source(path string) (string, error)
	// Warnings of the warm-up compilation.
	Warnings() []Diagnostic
}

// A registered template, ready for repeated compilation.
//
// Dropping it removes the template from the cache. Documents compiled from it stay
// usable.
type Template struct {
	ffiObject FfiObject
}

// Register a packed template and compile it once with the given inputs as a warm-up.
func NewTemplate(files []byte, jsonInputs map[string]string, blobInputs map[string]BlobInput, mode CompilationMode, limits *ZipLimits) (*Template, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_oicana_uniffi_fn_constructor_template_new(FfiConverterBytesINSTANCE.Lower(files), FfiConverterMapStringStringINSTANCE.Lower(jsonInputs), FfiConverterMapStringBlobInputINSTANCE.Lower(blobInputs), FfiConverterCompilationModeINSTANCE.Lower(mode), FfiConverterOptionalZipLimitsINSTANCE.Lower(limits), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *Template
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterTemplateINSTANCE.Lift(_uniffiRV), nil
	}
}

// Compile the template with the given inputs.
func (_self *Template) Compile(jsonInputs map[string]string, blobInputs map[string]BlobInput, mode CompilationMode) (*CompiledDocument, error) {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_oicana_uniffi_fn_method_template_compile(
			_pointer, FfiConverterMapStringStringINSTANCE.Lower(jsonInputs), FfiConverterMapStringBlobInputINSTANCE.Lower(blobInputs), FfiConverterCompilationModeINSTANCE.Lower(mode), _uniffiStatus)
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue *CompiledDocument
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterCompiledDocumentINSTANCE.Lift(_uniffiRV), nil
	}
}

// Raw bytes of a file inside the template.
func (_self *Template) File(path string) ([]byte, error) {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_template_file(
				_pointer, FfiConverterStringINSTANCE.Lower(path), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue []byte
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterBytesINSTANCE.Lift(_uniffiRV), nil
	}
}

// The template manifest as JSON.
func (_self *Template) Manifest() (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_template_manifest(
				_pointer, _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue string
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStringINSTANCE.Lift(_uniffiRV), nil
	}
}

// Enable or disable validating JSON inputs against their schemas. Enabled by default.
func (_self *Template) SetValidateInputs(validate bool) error {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	_, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_oicana_uniffi_fn_method_template_set_validate_inputs(
			_pointer, FfiConverterBoolINSTANCE.Lower(validate), _uniffiStatus)
		return false
	})
	return _uniffiErr.AsError()
}

// Source text of a file inside the template.
func (_self *Template) Source(path string) (string, error) {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_template_source(
				_pointer, FfiConverterStringINSTANCE.Lower(path), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue string
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterStringINSTANCE.Lift(_uniffiRV), nil
	}
}

// Warnings of the warm-up compilation.
func (_self *Template) Warnings() []Diagnostic {
	_pointer := _self.ffiObject.incrementPointer("*Template")
	defer _self.ffiObject.decrementPointer()
	return FfiConverterSequenceDiagnosticINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_method_template_warnings(
				_pointer, _uniffiStatus),
		}
	}))
}
func (object *Template) Destroy() {
	runtime.SetFinalizer(object, nil)
	object.ffiObject.destroy()
}

type FfiConverterTemplate struct{}

var FfiConverterTemplateINSTANCE = FfiConverterTemplate{}

func (c FfiConverterTemplate) Lift(handle C.uint64_t) *Template {
	result := &Template{
		newFfiObject(
			handle,
			func(handle C.uint64_t, status *C.RustCallStatus) C.uint64_t {
				return C.uniffi_oicana_uniffi_fn_clone_template(handle, status)
			},
			func(handle C.uint64_t, status *C.RustCallStatus) {
				C.uniffi_oicana_uniffi_fn_free_template(handle, status)
			},
		),
	}
	runtime.SetFinalizer(result, (*Template).Destroy)
	return result
}

func (c FfiConverterTemplate) Read(reader io.Reader) *Template {
	return c.Lift(C.uint64_t(readUint64(reader)))
}

func (c FfiConverterTemplate) Lower(value *Template) C.uint64_t {
	// TODO: this is bad - all synchronization from ObjectRuntime.go is discarded here,
	// because the handle will be decremented immediately after this function returns,
	// and someone will be left holding onto a non-locked handle.
	handle := value.ffiObject.incrementPointer("*Template")
	defer value.ffiObject.decrementPointer()
	return handle
}

func (c FfiConverterTemplate) Write(writer io.Writer, value *Template) {
	writeUint64(writer, uint64(c.Lower(value)))
}

func LiftFromExternalTemplate(handle uint64) *Template {
	return FfiConverterTemplateINSTANCE.Lift(C.uint64_t(handle))
}

func LowerToExternalTemplate(value *Template) uint64 {
	return uint64(FfiConverterTemplateINSTANCE.Lower(value))
}

type FfiDestroyerTemplate struct{}

func (_ FfiDestroyerTemplate) Destroy(value *Template) {
	value.Destroy()
}

// A blob input value.
type BlobInput struct {
	// Raw bytes of the blob.
	Data []byte
	// Metadata of the blob as a JSON object.
	Metadata string
}

func (r *BlobInput) Destroy() {
	FfiDestroyerBytes{}.Destroy(r.Data)
	FfiDestroyerString{}.Destroy(r.Metadata)
}

type FfiConverterBlobInput struct{}

var FfiConverterBlobInputINSTANCE = FfiConverterBlobInput{}

func (c FfiConverterBlobInput) Lift(rb RustBufferI) BlobInput {
	return LiftFromRustBuffer[BlobInput](c, rb)
}

func (c FfiConverterBlobInput) Read(reader io.Reader) BlobInput {
	return BlobInput{
		FfiConverterBytesINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterBlobInput) Lower(value BlobInput) C.RustBuffer {
	return LowerIntoRustBuffer[BlobInput](c, value)
}

func (c FfiConverterBlobInput) LowerExternal(value BlobInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[BlobInput](c, value))
}

func (c FfiConverterBlobInput) Write(writer io.Writer, value BlobInput) {
	FfiConverterBytesINSTANCE.Write(writer, value.Data)
	FfiConverterStringINSTANCE.Write(writer, value.Metadata)
}

type FfiDestroyerBlobInput struct{}

func (_ FfiDestroyerBlobInput) Destroy(value BlobInput) {
	value.Destroy()
}

// A compilation diagnostic, like a warning, with its location in the template.
type Diagnostic struct {
	// Whether this is an error or a warning.
	Severity DiagnosticSeverity
	// The message describing the problem.
	Message string
	// Where in the template the problem is, if it points into a file.
	Location *SourceLocation
	// Hints on how to address the problem.
	Hints []string
	// The function calls leading to the problem, innermost first.
	Trace []TracePoint
	// The diagnostic formatted for humans, with source excerpts and the configured
	// diagnostic color.
	Formatted string
}

func (r *Diagnostic) Destroy() {
	FfiDestroyerDiagnosticSeverity{}.Destroy(r.Severity)
	FfiDestroyerString{}.Destroy(r.Message)
	FfiDestroyerOptionalSourceLocation{}.Destroy(r.Location)
	FfiDestroyerSequenceString{}.Destroy(r.Hints)
	FfiDestroyerSequenceTracePoint{}.Destroy(r.Trace)
	FfiDestroyerString{}.Destroy(r.Formatted)
}

type FfiConverterDiagnostic struct{}

var FfiConverterDiagnosticINSTANCE = FfiConverterDiagnostic{}

func (c FfiConverterDiagnostic) Lift(rb RustBufferI) Diagnostic {
	return LiftFromRustBuffer[Diagnostic](c, rb)
}

func (c FfiConverterDiagnostic) Read(reader io.Reader) Diagnostic {
	return Diagnostic{
		FfiConverterDiagnosticSeverityINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalSourceLocationINSTANCE.Read(reader),
		FfiConverterSequenceStringINSTANCE.Read(reader),
		FfiConverterSequenceTracePointINSTANCE.Read(reader),
		FfiConverterStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterDiagnostic) Lower(value Diagnostic) C.RustBuffer {
	return LowerIntoRustBuffer[Diagnostic](c, value)
}

func (c FfiConverterDiagnostic) LowerExternal(value Diagnostic) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[Diagnostic](c, value))
}

func (c FfiConverterDiagnostic) Write(writer io.Writer, value Diagnostic) {
	FfiConverterDiagnosticSeverityINSTANCE.Write(writer, value.Severity)
	FfiConverterStringINSTANCE.Write(writer, value.Message)
	FfiConverterOptionalSourceLocationINSTANCE.Write(writer, value.Location)
	FfiConverterSequenceStringINSTANCE.Write(writer, value.Hints)
	FfiConverterSequenceTracePointINSTANCE.Write(writer, value.Trace)
	FfiConverterStringINSTANCE.Write(writer, value.Formatted)
}

type FfiDestroyerDiagnostic struct{}

func (_ FfiDestroyerDiagnostic) Destroy(value Diagnostic) {
	value.Destroy()
}

// An exported document with the warnings of its compilation.
type ExportOnceResult struct {
	// The exported document.
	Bytes []byte
	// Compilation warnings.
	Warnings []Diagnostic
}

func (r *ExportOnceResult) Destroy() {
	FfiDestroyerBytes{}.Destroy(r.Bytes)
	FfiDestroyerSequenceDiagnostic{}.Destroy(r.Warnings)
}

type FfiConverterExportOnceResult struct{}

var FfiConverterExportOnceResultINSTANCE = FfiConverterExportOnceResult{}

func (c FfiConverterExportOnceResult) Lift(rb RustBufferI) ExportOnceResult {
	return LiftFromRustBuffer[ExportOnceResult](c, rb)
}

func (c FfiConverterExportOnceResult) Read(reader io.Reader) ExportOnceResult {
	return ExportOnceResult{
		FfiConverterBytesINSTANCE.Read(reader),
		FfiConverterSequenceDiagnosticINSTANCE.Read(reader),
	}
}

func (c FfiConverterExportOnceResult) Lower(value ExportOnceResult) C.RustBuffer {
	return LowerIntoRustBuffer[ExportOnceResult](c, value)
}

func (c FfiConverterExportOnceResult) LowerExternal(value ExportOnceResult) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ExportOnceResult](c, value))
}

func (c FfiConverterExportOnceResult) Write(writer io.Writer, value ExportOnceResult) {
	FfiConverterBytesINSTANCE.Write(writer, value.Bytes)
	FfiConverterSequenceDiagnosticINSTANCE.Write(writer, value.Warnings)
}

type FfiDestroyerExportOnceResult struct{}

func (_ FfiDestroyerExportOnceResult) Destroy(value ExportOnceResult) {
	value.Destroy()
}

// A contiguous range of pages, with 0-based inclusive bounds.
type PageRange struct {
	// First page to export. `None` starts at the first page.
	Start *uint64
	// Last page to export. `None` ends at the last page.
	End *uint64
}

func (r *PageRange) Destroy() {
	FfiDestroyerOptionalUint64{}.Destroy(r.Start)
	FfiDestroyerOptionalUint64{}.Destroy(r.End)
}

type FfiConverterPageRange struct{}

var FfiConverterPageRangeINSTANCE = FfiConverterPageRange{}

func (c FfiConverterPageRange) Lift(rb RustBufferI) PageRange {
	return LiftFromRustBuffer[PageRange](c, rb)
}

func (c FfiConverterPageRange) Read(reader io.Reader) PageRange {
	return PageRange{
		FfiConverterOptionalUint64INSTANCE.Read(reader),
		FfiConverterOptionalUint64INSTANCE.Read(reader),
	}
}

func (c FfiConverterPageRange) Lower(value PageRange) C.RustBuffer {
	return LowerIntoRustBuffer[PageRange](c, value)
}

func (c FfiConverterPageRange) LowerExternal(value PageRange) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[PageRange](c, value))
}

func (c FfiConverterPageRange) Write(writer io.Writer, value PageRange) {
	FfiConverterOptionalUint64INSTANCE.Write(writer, value.Start)
	FfiConverterOptionalUint64INSTANCE.Write(writer, value.End)
}

type FfiDestroyerPageRange struct{}

func (_ FfiDestroyerPageRange) Destroy(value PageRange) {
	value.Destroy()
}

// Size of a page in typographic points.
type PageSize struct {
	// Page width.
	Width float64
	// Page height.
	Height float64
}

func (r *PageSize) Destroy() {
	FfiDestroyerFloat64{}.Destroy(r.Width)
	FfiDestroyerFloat64{}.Destroy(r.Height)
}

type FfiConverterPageSize struct{}

var FfiConverterPageSizeINSTANCE = FfiConverterPageSize{}

func (c FfiConverterPageSize) Lift(rb RustBufferI) PageSize {
	return LiftFromRustBuffer[PageSize](c, rb)
}

func (c FfiConverterPageSize) Read(reader io.Reader) PageSize {
	return PageSize{
		FfiConverterFloat64INSTANCE.Read(reader),
		FfiConverterFloat64INSTANCE.Read(reader),
	}
}

func (c FfiConverterPageSize) Lower(value PageSize) C.RustBuffer {
	return LowerIntoRustBuffer[PageSize](c, value)
}

func (c FfiConverterPageSize) LowerExternal(value PageSize) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[PageSize](c, value))
}

func (c FfiConverterPageSize) Write(writer io.Writer, value PageSize) {
	FfiConverterFloat64INSTANCE.Write(writer, value.Width)
	FfiConverterFloat64INSTANCE.Write(writer, value.Height)
}

type FfiDestroyerPageSize struct{}

func (_ FfiDestroyerPageSize) Destroy(value PageSize) {
	value.Destroy()
}

// A font face registered by the host.
type RegisteredFont struct {
	// The family name.
	Family string
	// The file the face was read from, if it was registered by path.
	Path *string
}

func (r *RegisteredFont) Destroy() {
	FfiDestroyerString{}.Destroy(r.Family)
	FfiDestroyerOptionalString{}.Destroy(r.Path)
}

type FfiConverterRegisteredFont struct{}

var FfiConverterRegisteredFontINSTANCE = FfiConverterRegisteredFont{}

func (c FfiConverterRegisteredFont) Lift(rb RustBufferI) RegisteredFont {
	return LiftFromRustBuffer[RegisteredFont](c, rb)
}

func (c FfiConverterRegisteredFont) Read(reader io.Reader) RegisteredFont {
	return RegisteredFont{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalStringINSTANCE.Read(reader),
	}
}

func (c FfiConverterRegisteredFont) Lower(value RegisteredFont) C.RustBuffer {
	return LowerIntoRustBuffer[RegisteredFont](c, value)
}

func (c FfiConverterRegisteredFont) LowerExternal(value RegisteredFont) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[RegisteredFont](c, value))
}

func (c FfiConverterRegisteredFont) Write(writer io.Writer, value RegisteredFont) {
	FfiConverterStringINSTANCE.Write(writer, value.Family)
	FfiConverterOptionalStringINSTANCE.Write(writer, value.Path)
}

type FfiDestroyerRegisteredFont struct{}

func (_ FfiDestroyerRegisteredFont) Destroy(value RegisteredFont) {
	value.Destroy()
}

// A range in a file of the template.
//
// Lines and columns are 1-based. Columns count characters.
type SourceLocation struct {
	// Path of the file in the template, like `/main.typ`.
	// Files of packages are prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
	File string
	// Line of the start of the range.
	Line uint32
	// Column of the start of the range.
	Column uint32
	// Line of the end of the range.
	EndLine uint32
	// Column of the end of the range (exclusive).
	EndColumn uint32
}

func (r *SourceLocation) Destroy() {
	FfiDestroyerString{}.Destroy(r.File)
	FfiDestroyerUint32{}.Destroy(r.Line)
	FfiDestroyerUint32{}.Destroy(r.Column)
	FfiDestroyerUint32{}.Destroy(r.EndLine)
	FfiDestroyerUint32{}.Destroy(r.EndColumn)
}

type FfiConverterSourceLocation struct{}

var FfiConverterSourceLocationINSTANCE = FfiConverterSourceLocation{}

func (c FfiConverterSourceLocation) Lift(rb RustBufferI) SourceLocation {
	return LiftFromRustBuffer[SourceLocation](c, rb)
}

func (c FfiConverterSourceLocation) Read(reader io.Reader) SourceLocation {
	return SourceLocation{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterUint32INSTANCE.Read(reader),
		FfiConverterUint32INSTANCE.Read(reader),
		FfiConverterUint32INSTANCE.Read(reader),
		FfiConverterUint32INSTANCE.Read(reader),
	}
}

func (c FfiConverterSourceLocation) Lower(value SourceLocation) C.RustBuffer {
	return LowerIntoRustBuffer[SourceLocation](c, value)
}

func (c FfiConverterSourceLocation) LowerExternal(value SourceLocation) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[SourceLocation](c, value))
}

func (c FfiConverterSourceLocation) Write(writer io.Writer, value SourceLocation) {
	FfiConverterStringINSTANCE.Write(writer, value.File)
	FfiConverterUint32INSTANCE.Write(writer, value.Line)
	FfiConverterUint32INSTANCE.Write(writer, value.Column)
	FfiConverterUint32INSTANCE.Write(writer, value.EndLine)
	FfiConverterUint32INSTANCE.Write(writer, value.EndColumn)
}

type FfiDestroyerSourceLocation struct{}

func (_ FfiDestroyerSourceLocation) Destroy(value SourceLocation) {
	value.Destroy()
}

// A step in the function calls leading to a [`Diagnostic`].
type TracePoint struct {
	// What happened at this step, like `error occurred in this function call`.
	Message string
	// Where the step happened, if it points into a file.
	Location *SourceLocation
}

func (r *TracePoint) Destroy() {
	FfiDestroyerString{}.Destroy(r.Message)
	FfiDestroyerOptionalSourceLocation{}.Destroy(r.Location)
}

type FfiConverterTracePoint struct{}

var FfiConverterTracePointINSTANCE = FfiConverterTracePoint{}

func (c FfiConverterTracePoint) Lift(rb RustBufferI) TracePoint {
	return LiftFromRustBuffer[TracePoint](c, rb)
}

func (c FfiConverterTracePoint) Read(reader io.Reader) TracePoint {
	return TracePoint{
		FfiConverterStringINSTANCE.Read(reader),
		FfiConverterOptionalSourceLocationINSTANCE.Read(reader),
	}
}

func (c FfiConverterTracePoint) Lower(value TracePoint) C.RustBuffer {
	return LowerIntoRustBuffer[TracePoint](c, value)
}

func (c FfiConverterTracePoint) LowerExternal(value TracePoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[TracePoint](c, value))
}

func (c FfiConverterTracePoint) Write(writer io.Writer, value TracePoint) {
	FfiConverterStringINSTANCE.Write(writer, value.Message)
	FfiConverterOptionalSourceLocationINSTANCE.Write(writer, value.Location)
}

type FfiDestroyerTracePoint struct{}

func (_ FfiDestroyerTracePoint) Destroy(value TracePoint) {
	value.Destroy()
}

// Limits for reading a packed template.
type ZipLimits struct {
	// Maximum number of zip entries. `None` keeps the default.
	MaxEntries *uint64
	// Maximum total decompressed size in bytes. `None` keeps the default.
	MaxTotalDecompressedBytes *uint64
}

func (r *ZipLimits) Destroy() {
	FfiDestroyerOptionalUint64{}.Destroy(r.MaxEntries)
	FfiDestroyerOptionalUint64{}.Destroy(r.MaxTotalDecompressedBytes)
}

type FfiConverterZipLimits struct{}

var FfiConverterZipLimitsINSTANCE = FfiConverterZipLimits{}

func (c FfiConverterZipLimits) Lift(rb RustBufferI) ZipLimits {
	return LiftFromRustBuffer[ZipLimits](c, rb)
}

func (c FfiConverterZipLimits) Read(reader io.Reader) ZipLimits {
	return ZipLimits{
		FfiConverterOptionalUint64INSTANCE.Read(reader),
		FfiConverterOptionalUint64INSTANCE.Read(reader),
	}
}

func (c FfiConverterZipLimits) Lower(value ZipLimits) C.RustBuffer {
	return LowerIntoRustBuffer[ZipLimits](c, value)
}

func (c FfiConverterZipLimits) LowerExternal(value ZipLimits) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ZipLimits](c, value))
}

func (c FfiConverterZipLimits) Write(writer io.Writer, value ZipLimits) {
	FfiConverterOptionalUint64INSTANCE.Write(writer, value.MaxEntries)
	FfiConverterOptionalUint64INSTANCE.Write(writer, value.MaxTotalDecompressedBytes)
}

type FfiDestroyerZipLimits struct{}

func (_ FfiDestroyerZipLimits) Destroy(value ZipLimits) {
	value.Destroy()
}

// How required inputs without a value are resolved.
type CompilationMode uint

const (
	// Fall back to the development value, then the default value.
	CompilationModeDevelopment CompilationMode = 1
	// Fall back to the default value.
	CompilationModeProduction CompilationMode = 2
)

type FfiConverterCompilationMode struct{}

var FfiConverterCompilationModeINSTANCE = FfiConverterCompilationMode{}

func (c FfiConverterCompilationMode) Lift(rb RustBufferI) CompilationMode {
	return LiftFromRustBuffer[CompilationMode](c, rb)
}

func (c FfiConverterCompilationMode) Lower(value CompilationMode) C.RustBuffer {
	return LowerIntoRustBuffer[CompilationMode](c, value)
}

func (c FfiConverterCompilationMode) LowerExternal(value CompilationMode) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[CompilationMode](c, value))
}
func (FfiConverterCompilationMode) Read(reader io.Reader) CompilationMode {
	id := readInt32(reader)
	return CompilationMode(id)
}

func (FfiConverterCompilationMode) Write(writer io.Writer, value CompilationMode) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerCompilationMode struct{}

func (_ FfiDestroyerCompilationMode) Destroy(value CompilationMode) {
}

// Coloring of compilation diagnostics.
type DiagnosticColor uint

const (
	// Plain text.
	DiagnosticColorNone DiagnosticColor = 1
	// ANSI escape codes.
	DiagnosticColorAnsi DiagnosticColor = 2
)

type FfiConverterDiagnosticColor struct{}

var FfiConverterDiagnosticColorINSTANCE = FfiConverterDiagnosticColor{}

func (c FfiConverterDiagnosticColor) Lift(rb RustBufferI) DiagnosticColor {
	return LiftFromRustBuffer[DiagnosticColor](c, rb)
}

func (c FfiConverterDiagnosticColor) Lower(value DiagnosticColor) C.RustBuffer {
	return LowerIntoRustBuffer[DiagnosticColor](c, value)
}

func (c FfiConverterDiagnosticColor) LowerExternal(value DiagnosticColor) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DiagnosticColor](c, value))
}
func (FfiConverterDiagnosticColor) Read(reader io.Reader) DiagnosticColor {
	id := readInt32(reader)
	return DiagnosticColor(id)
}

func (FfiConverterDiagnosticColor) Write(writer io.Writer, value DiagnosticColor) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerDiagnosticColor struct{}

func (_ FfiDestroyerDiagnosticColor) Destroy(value DiagnosticColor) {
}

// The severity of a [`Diagnostic`].
type DiagnosticSeverity uint

const (
	// A fatal error.
	DiagnosticSeverityError DiagnosticSeverity = 1
	// A non-fatal warning.
	DiagnosticSeverityWarning DiagnosticSeverity = 2
)

type FfiConverterDiagnosticSeverity struct{}

var FfiConverterDiagnosticSeverityINSTANCE = FfiConverterDiagnosticSeverity{}

func (c FfiConverterDiagnosticSeverity) Lift(rb RustBufferI) DiagnosticSeverity {
	return LiftFromRustBuffer[DiagnosticSeverity](c, rb)
}

func (c FfiConverterDiagnosticSeverity) Lower(value DiagnosticSeverity) C.RustBuffer {
	return LowerIntoRustBuffer[DiagnosticSeverity](c, value)
}

func (c FfiConverterDiagnosticSeverity) LowerExternal(value DiagnosticSeverity) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[DiagnosticSeverity](c, value))
}
func (FfiConverterDiagnosticSeverity) Read(reader io.Reader) DiagnosticSeverity {
	id := readInt32(reader)
	return DiagnosticSeverity(id)
}

func (FfiConverterDiagnosticSeverity) Write(writer io.Writer, value DiagnosticSeverity) {
	writeInt32(writer, int32(value))
}

type FfiDestroyerDiagnosticSeverity struct{}

func (_ FfiDestroyerDiagnosticSeverity) Destroy(value DiagnosticSeverity) {
}

// Format of an exported document.
type ExportFormat interface {
	Destroy()
}

// PDF, using the standards from the template manifest.
type ExportFormatPdf struct {
}

func (e ExportFormatPdf) Destroy() {
}

// PNG image.
type ExportFormatPng struct {
	PixelsPerPt float32
}

func (e ExportFormatPng) Destroy() {
	FfiDestroyerFloat32{}.Destroy(e.PixelsPerPt)
}

// SVG image.
type ExportFormatSvg struct {
}

func (e ExportFormatSvg) Destroy() {
}

type FfiConverterExportFormat struct{}

var FfiConverterExportFormatINSTANCE = FfiConverterExportFormat{}

func (c FfiConverterExportFormat) Lift(rb RustBufferI) ExportFormat {
	return LiftFromRustBuffer[ExportFormat](c, rb)
}

func (c FfiConverterExportFormat) Lower(value ExportFormat) C.RustBuffer {
	return LowerIntoRustBuffer[ExportFormat](c, value)
}

func (c FfiConverterExportFormat) LowerExternal(value ExportFormat) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[ExportFormat](c, value))
}
func (FfiConverterExportFormat) Read(reader io.Reader) ExportFormat {
	id := readInt32(reader)
	switch id {
	case 1:
		return ExportFormatPdf{}
	case 2:
		return ExportFormatPng{
			FfiConverterFloat32INSTANCE.Read(reader),
		}
	case 3:
		return ExportFormatSvg{}
	default:
		panic(fmt.Sprintf("invalid enum value %v in FfiConverterExportFormat.Read()", id))
	}
}

func (FfiConverterExportFormat) Write(writer io.Writer, value ExportFormat) {
	switch variant_value := value.(type) {
	case ExportFormatPdf:
		writeInt32(writer, 1)
	case ExportFormatPng:
		writeInt32(writer, 2)
		FfiConverterFloat32INSTANCE.Write(writer, variant_value.PixelsPerPt)
	case ExportFormatSvg:
		writeInt32(writer, 3)
	default:
		_ = variant_value
		panic(fmt.Sprintf("invalid enum value `%v` in FfiConverterExportFormat.Write", value))
	}
}

type FfiDestroyerExportFormat struct{}

func (_ FfiDestroyerExportFormat) Destroy(value ExportFormat) {
	value.Destroy()
}

// Errors raised by Oicana.
//
// Each variant carries the human-readable message, including rendered compilation
// diagnostics.
type OicanaError struct {
	err error
}

// Convenience method to turn *OicanaError into error
// Avoiding treating nil pointer as non nil error interface
func (err *OicanaError) AsError() error {
	if err == nil {
		return nil
	} else {
		return err
	}
}

func (err OicanaError) Error() string {
	return fmt.Sprintf("OicanaError: %s", err.err.Error())
}

func (err OicanaError) Unwrap() error {
	return err.err
}

// Err* are used for checking error type with `errors.Is`
var ErrOicanaErrorInvalidTemplate = fmt.Errorf("OicanaErrorInvalidTemplate")
var ErrOicanaErrorInvalidInput = fmt.Errorf("OicanaErrorInvalidInput")
var ErrOicanaErrorCompilation = fmt.Errorf("OicanaErrorCompilation")
var ErrOicanaErrorFileNotFound = fmt.Errorf("OicanaErrorFileNotFound")
var ErrOicanaErrorExport = fmt.Errorf("OicanaErrorExport")
var ErrOicanaErrorNotFound = fmt.Errorf("OicanaErrorNotFound")
var ErrOicanaErrorInternal = fmt.Errorf("OicanaErrorInternal")

// Variant structs
// The packed template could not be read or its manifest is invalid.
type OicanaErrorInvalidTemplate struct {
	message string
}

// The packed template could not be read or its manifest is invalid.
func NewOicanaErrorInvalidTemplate() *OicanaError {
	return &OicanaError{err: &OicanaErrorInvalidTemplate{}}
}

func (e OicanaErrorInvalidTemplate) destroy() {
}

func (err OicanaErrorInvalidTemplate) Error() string {
	return fmt.Sprintf("InvalidTemplate: %s", err.message)
}

func (self OicanaErrorInvalidTemplate) Is(target error) bool {
	return target == ErrOicanaErrorInvalidTemplate
}

// An input value, its blob metadata, or a page range is invalid.
type OicanaErrorInvalidInput struct {
	message string
}

// An input value, its blob metadata, or a page range is invalid.
func NewOicanaErrorInvalidInput() *OicanaError {
	return &OicanaError{err: &OicanaErrorInvalidInput{}}
}

func (e OicanaErrorInvalidInput) destroy() {
}

func (err OicanaErrorInvalidInput) Error() string {
	return fmt.Sprintf("InvalidInput: %s", err.message)
}

func (self OicanaErrorInvalidInput) Is(target error) bool {
	return target == ErrOicanaErrorInvalidInput
}

// Compiling the template failed.
type OicanaErrorCompilation struct {
	message string
}

// Compiling the template failed.
func NewOicanaErrorCompilation() *OicanaError {
	return &OicanaError{err: &OicanaErrorCompilation{}}
}

func (e OicanaErrorCompilation) destroy() {
}

func (err OicanaErrorCompilation) Error() string {
	return fmt.Sprintf("Compilation: %s", err.message)
}

func (self OicanaErrorCompilation) Is(target error) bool {
	return target == ErrOicanaErrorCompilation
}

// A file requested from the template could not be loaded.
type OicanaErrorFileNotFound struct {
	message string
}

// A file requested from the template could not be loaded.
func NewOicanaErrorFileNotFound() *OicanaError {
	return &OicanaError{err: &OicanaErrorFileNotFound{}}
}

func (e OicanaErrorFileNotFound) destroy() {
}

func (err OicanaErrorFileNotFound) Error() string {
	return fmt.Sprintf("FileNotFound: %s", err.message)
}

func (self OicanaErrorFileNotFound) Is(target error) bool {
	return target == ErrOicanaErrorFileNotFound
}

// Encoding the document to the requested format failed.
type OicanaErrorExport struct {
	message string
}

// Encoding the document to the requested format failed.
func NewOicanaErrorExport() *OicanaError {
	return &OicanaError{err: &OicanaErrorExport{}}
}

func (e OicanaErrorExport) destroy() {
}

func (err OicanaErrorExport) Error() string {
	return fmt.Sprintf("Export: %s", err.message)
}

func (self OicanaErrorExport) Is(target error) bool {
	return target == ErrOicanaErrorExport
}

// A template or document is no longer registered.
type OicanaErrorNotFound struct {
	message string
}

// A template or document is no longer registered.
func NewOicanaErrorNotFound() *OicanaError {
	return &OicanaError{err: &OicanaErrorNotFound{}}
}

func (e OicanaErrorNotFound) destroy() {
}

func (err OicanaErrorNotFound) Error() string {
	return fmt.Sprintf("NotFound: %s", err.message)
}

func (self OicanaErrorNotFound) Is(target error) bool {
	return target == ErrOicanaErrorNotFound
}

// An unexpected internal failure.
type OicanaErrorInternal struct {
	message string
}

// An unexpected internal failure.
func NewOicanaErrorInternal() *OicanaError {
	return &OicanaError{err: &OicanaErrorInternal{}}
}

func (e OicanaErrorInternal) destroy() {
}

func (err OicanaErrorInternal) Error() string {
	return fmt.Sprintf("Internal: %s", err.message)
}

func (self OicanaErrorInternal) Is(target error) bool {
	return target == ErrOicanaErrorInternal
}

type FfiConverterOicanaError struct{}

var FfiConverterOicanaErrorINSTANCE = FfiConverterOicanaError{}

func (c FfiConverterOicanaError) Lift(eb RustBufferI) *OicanaError {
	return LiftFromRustBuffer[*OicanaError](c, eb)
}

func (c FfiConverterOicanaError) Lower(value *OicanaError) C.RustBuffer {
	return LowerIntoRustBuffer[*OicanaError](c, value)
}

func (c FfiConverterOicanaError) LowerExternal(value *OicanaError) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*OicanaError](c, value))
}

func (c FfiConverterOicanaError) Read(reader io.Reader) *OicanaError {
	errorID := readUint32(reader)

	message := FfiConverterStringINSTANCE.Read(reader)
	switch errorID {
	case 1:
		return &OicanaError{&OicanaErrorInvalidTemplate{message}}
	case 2:
		return &OicanaError{&OicanaErrorInvalidInput{message}}
	case 3:
		return &OicanaError{&OicanaErrorCompilation{message}}
	case 4:
		return &OicanaError{&OicanaErrorFileNotFound{message}}
	case 5:
		return &OicanaError{&OicanaErrorExport{message}}
	case 6:
		return &OicanaError{&OicanaErrorNotFound{message}}
	case 7:
		return &OicanaError{&OicanaErrorInternal{message}}
	default:
		panic(fmt.Sprintf("Unknown error code %d in FfiConverterOicanaError.Read()", errorID))
	}

}

func (c FfiConverterOicanaError) Write(writer io.Writer, value *OicanaError) {
	switch variantValue := value.err.(type) {
	case *OicanaErrorInvalidTemplate:
		writeInt32(writer, 1)
	case *OicanaErrorInvalidInput:
		writeInt32(writer, 2)
	case *OicanaErrorCompilation:
		writeInt32(writer, 3)
	case *OicanaErrorFileNotFound:
		writeInt32(writer, 4)
	case *OicanaErrorExport:
		writeInt32(writer, 5)
	case *OicanaErrorNotFound:
		writeInt32(writer, 6)
	case *OicanaErrorInternal:
		writeInt32(writer, 7)
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiConverterOicanaError.Write", value))
	}
}

type FfiDestroyerOicanaError struct{}

func (_ FfiDestroyerOicanaError) Destroy(value *OicanaError) {
	switch variantValue := value.err.(type) {
	case OicanaErrorInvalidTemplate:
		variantValue.destroy()
	case OicanaErrorInvalidInput:
		variantValue.destroy()
	case OicanaErrorCompilation:
		variantValue.destroy()
	case OicanaErrorFileNotFound:
		variantValue.destroy()
	case OicanaErrorExport:
		variantValue.destroy()
	case OicanaErrorNotFound:
		variantValue.destroy()
	case OicanaErrorInternal:
		variantValue.destroy()
	default:
		_ = variantValue
		panic(fmt.Sprintf("invalid error value `%v` in FfiDestroyerOicanaError.Destroy", value))
	}
}

type FfiConverterOptionalUint64 struct{}

var FfiConverterOptionalUint64INSTANCE = FfiConverterOptionalUint64{}

func (c FfiConverterOptionalUint64) Lift(rb RustBufferI) *uint64 {
	return LiftFromRustBuffer[*uint64](c, rb)
}

func (_ FfiConverterOptionalUint64) Read(reader io.Reader) *uint64 {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterUint64INSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalUint64) Lower(value *uint64) C.RustBuffer {
	return LowerIntoRustBuffer[*uint64](c, value)
}

func (c FfiConverterOptionalUint64) LowerExternal(value *uint64) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*uint64](c, value))
}

func (_ FfiConverterOptionalUint64) Write(writer io.Writer, value *uint64) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterUint64INSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalUint64 struct{}

func (_ FfiDestroyerOptionalUint64) Destroy(value *uint64) {
	if value != nil {
		FfiDestroyerUint64{}.Destroy(*value)
	}
}

type FfiConverterOptionalString struct{}

var FfiConverterOptionalStringINSTANCE = FfiConverterOptionalString{}

func (c FfiConverterOptionalString) Lift(rb RustBufferI) *string {
	return LiftFromRustBuffer[*string](c, rb)
}

func (_ FfiConverterOptionalString) Read(reader io.Reader) *string {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterStringINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalString) Lower(value *string) C.RustBuffer {
	return LowerIntoRustBuffer[*string](c, value)
}

func (c FfiConverterOptionalString) LowerExternal(value *string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*string](c, value))
}

func (_ FfiConverterOptionalString) Write(writer io.Writer, value *string) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterStringINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalString struct{}

func (_ FfiDestroyerOptionalString) Destroy(value *string) {
	if value != nil {
		FfiDestroyerString{}.Destroy(*value)
	}
}

type FfiConverterOptionalPageRange struct{}

var FfiConverterOptionalPageRangeINSTANCE = FfiConverterOptionalPageRange{}

func (c FfiConverterOptionalPageRange) Lift(rb RustBufferI) *PageRange {
	return LiftFromRustBuffer[*PageRange](c, rb)
}

func (_ FfiConverterOptionalPageRange) Read(reader io.Reader) *PageRange {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterPageRangeINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalPageRange) Lower(value *PageRange) C.RustBuffer {
	return LowerIntoRustBuffer[*PageRange](c, value)
}

func (c FfiConverterOptionalPageRange) LowerExternal(value *PageRange) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*PageRange](c, value))
}

func (_ FfiConverterOptionalPageRange) Write(writer io.Writer, value *PageRange) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterPageRangeINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalPageRange struct{}

func (_ FfiDestroyerOptionalPageRange) Destroy(value *PageRange) {
	if value != nil {
		FfiDestroyerPageRange{}.Destroy(*value)
	}
}

type FfiConverterOptionalSourceLocation struct{}

var FfiConverterOptionalSourceLocationINSTANCE = FfiConverterOptionalSourceLocation{}

func (c FfiConverterOptionalSourceLocation) Lift(rb RustBufferI) *SourceLocation {
	return LiftFromRustBuffer[*SourceLocation](c, rb)
}

func (_ FfiConverterOptionalSourceLocation) Read(reader io.Reader) *SourceLocation {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterSourceLocationINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalSourceLocation) Lower(value *SourceLocation) C.RustBuffer {
	return LowerIntoRustBuffer[*SourceLocation](c, value)
}

func (c FfiConverterOptionalSourceLocation) LowerExternal(value *SourceLocation) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*SourceLocation](c, value))
}

func (_ FfiConverterOptionalSourceLocation) Write(writer io.Writer, value *SourceLocation) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterSourceLocationINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalSourceLocation struct{}

func (_ FfiDestroyerOptionalSourceLocation) Destroy(value *SourceLocation) {
	if value != nil {
		FfiDestroyerSourceLocation{}.Destroy(*value)
	}
}

type FfiConverterOptionalZipLimits struct{}

var FfiConverterOptionalZipLimitsINSTANCE = FfiConverterOptionalZipLimits{}

func (c FfiConverterOptionalZipLimits) Lift(rb RustBufferI) *ZipLimits {
	return LiftFromRustBuffer[*ZipLimits](c, rb)
}

func (_ FfiConverterOptionalZipLimits) Read(reader io.Reader) *ZipLimits {
	if readInt8(reader) == 0 {
		return nil
	}
	temp := FfiConverterZipLimitsINSTANCE.Read(reader)
	return &temp
}

func (c FfiConverterOptionalZipLimits) Lower(value *ZipLimits) C.RustBuffer {
	return LowerIntoRustBuffer[*ZipLimits](c, value)
}

func (c FfiConverterOptionalZipLimits) LowerExternal(value *ZipLimits) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[*ZipLimits](c, value))
}

func (_ FfiConverterOptionalZipLimits) Write(writer io.Writer, value *ZipLimits) {
	if value == nil {
		writeInt8(writer, 0)
	} else {
		writeInt8(writer, 1)
		FfiConverterZipLimitsINSTANCE.Write(writer, *value)
	}
}

type FfiDestroyerOptionalZipLimits struct{}

func (_ FfiDestroyerOptionalZipLimits) Destroy(value *ZipLimits) {
	if value != nil {
		FfiDestroyerZipLimits{}.Destroy(*value)
	}
}

type FfiConverterSequenceString struct{}

var FfiConverterSequenceStringINSTANCE = FfiConverterSequenceString{}

func (c FfiConverterSequenceString) Lift(rb RustBufferI) []string {
	return LiftFromRustBuffer[[]string](c, rb)
}

func (c FfiConverterSequenceString) Read(reader io.Reader) []string {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]string, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterStringINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceString) Lower(value []string) C.RustBuffer {
	return LowerIntoRustBuffer[[]string](c, value)
}

func (c FfiConverterSequenceString) LowerExternal(value []string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]string](c, value))
}

func (c FfiConverterSequenceString) Write(writer io.Writer, value []string) {
	if len(value) > math.MaxInt32 {
		panic("[]string is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterStringINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceString struct{}

func (FfiDestroyerSequenceString) Destroy(sequence []string) {
	for _, value := range sequence {
		FfiDestroyerString{}.Destroy(value)
	}
}

type FfiConverterSequenceBytes struct{}

var FfiConverterSequenceBytesINSTANCE = FfiConverterSequenceBytes{}

func (c FfiConverterSequenceBytes) Lift(rb RustBufferI) [][]byte {
	return LiftFromRustBuffer[[][]byte](c, rb)
}

func (c FfiConverterSequenceBytes) Read(reader io.Reader) [][]byte {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([][]byte, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterBytesINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceBytes) Lower(value [][]byte) C.RustBuffer {
	return LowerIntoRustBuffer[[][]byte](c, value)
}

func (c FfiConverterSequenceBytes) LowerExternal(value [][]byte) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[][]byte](c, value))
}

func (c FfiConverterSequenceBytes) Write(writer io.Writer, value [][]byte) {
	if len(value) > math.MaxInt32 {
		panic("[][]byte is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterBytesINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceBytes struct{}

func (FfiDestroyerSequenceBytes) Destroy(sequence [][]byte) {
	for _, value := range sequence {
		FfiDestroyerBytes{}.Destroy(value)
	}
}

type FfiConverterSequenceDiagnostic struct{}

var FfiConverterSequenceDiagnosticINSTANCE = FfiConverterSequenceDiagnostic{}

func (c FfiConverterSequenceDiagnostic) Lift(rb RustBufferI) []Diagnostic {
	return LiftFromRustBuffer[[]Diagnostic](c, rb)
}

func (c FfiConverterSequenceDiagnostic) Read(reader io.Reader) []Diagnostic {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]Diagnostic, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterDiagnosticINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceDiagnostic) Lower(value []Diagnostic) C.RustBuffer {
	return LowerIntoRustBuffer[[]Diagnostic](c, value)
}

func (c FfiConverterSequenceDiagnostic) LowerExternal(value []Diagnostic) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]Diagnostic](c, value))
}

func (c FfiConverterSequenceDiagnostic) Write(writer io.Writer, value []Diagnostic) {
	if len(value) > math.MaxInt32 {
		panic("[]Diagnostic is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterDiagnosticINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceDiagnostic struct{}

func (FfiDestroyerSequenceDiagnostic) Destroy(sequence []Diagnostic) {
	for _, value := range sequence {
		FfiDestroyerDiagnostic{}.Destroy(value)
	}
}

type FfiConverterSequencePageSize struct{}

var FfiConverterSequencePageSizeINSTANCE = FfiConverterSequencePageSize{}

func (c FfiConverterSequencePageSize) Lift(rb RustBufferI) []PageSize {
	return LiftFromRustBuffer[[]PageSize](c, rb)
}

func (c FfiConverterSequencePageSize) Read(reader io.Reader) []PageSize {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]PageSize, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterPageSizeINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequencePageSize) Lower(value []PageSize) C.RustBuffer {
	return LowerIntoRustBuffer[[]PageSize](c, value)
}

func (c FfiConverterSequencePageSize) LowerExternal(value []PageSize) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]PageSize](c, value))
}

func (c FfiConverterSequencePageSize) Write(writer io.Writer, value []PageSize) {
	if len(value) > math.MaxInt32 {
		panic("[]PageSize is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterPageSizeINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequencePageSize struct{}

func (FfiDestroyerSequencePageSize) Destroy(sequence []PageSize) {
	for _, value := range sequence {
		FfiDestroyerPageSize{}.Destroy(value)
	}
}

type FfiConverterSequenceRegisteredFont struct{}

var FfiConverterSequenceRegisteredFontINSTANCE = FfiConverterSequenceRegisteredFont{}

func (c FfiConverterSequenceRegisteredFont) Lift(rb RustBufferI) []RegisteredFont {
	return LiftFromRustBuffer[[]RegisteredFont](c, rb)
}

func (c FfiConverterSequenceRegisteredFont) Read(reader io.Reader) []RegisteredFont {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]RegisteredFont, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterRegisteredFontINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceRegisteredFont) Lower(value []RegisteredFont) C.RustBuffer {
	return LowerIntoRustBuffer[[]RegisteredFont](c, value)
}

func (c FfiConverterSequenceRegisteredFont) LowerExternal(value []RegisteredFont) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]RegisteredFont](c, value))
}

func (c FfiConverterSequenceRegisteredFont) Write(writer io.Writer, value []RegisteredFont) {
	if len(value) > math.MaxInt32 {
		panic("[]RegisteredFont is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterRegisteredFontINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceRegisteredFont struct{}

func (FfiDestroyerSequenceRegisteredFont) Destroy(sequence []RegisteredFont) {
	for _, value := range sequence {
		FfiDestroyerRegisteredFont{}.Destroy(value)
	}
}

type FfiConverterSequenceTracePoint struct{}

var FfiConverterSequenceTracePointINSTANCE = FfiConverterSequenceTracePoint{}

func (c FfiConverterSequenceTracePoint) Lift(rb RustBufferI) []TracePoint {
	return LiftFromRustBuffer[[]TracePoint](c, rb)
}

func (c FfiConverterSequenceTracePoint) Read(reader io.Reader) []TracePoint {
	length := readInt32(reader)
	if length == 0 {
		return nil
	}
	result := make([]TracePoint, 0, length)
	for i := int32(0); i < length; i++ {
		result = append(result, FfiConverterTracePointINSTANCE.Read(reader))
	}
	return result
}

func (c FfiConverterSequenceTracePoint) Lower(value []TracePoint) C.RustBuffer {
	return LowerIntoRustBuffer[[]TracePoint](c, value)
}

func (c FfiConverterSequenceTracePoint) LowerExternal(value []TracePoint) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[[]TracePoint](c, value))
}

func (c FfiConverterSequenceTracePoint) Write(writer io.Writer, value []TracePoint) {
	if len(value) > math.MaxInt32 {
		panic("[]TracePoint is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(value)))
	for _, item := range value {
		FfiConverterTracePointINSTANCE.Write(writer, item)
	}
}

type FfiDestroyerSequenceTracePoint struct{}

func (FfiDestroyerSequenceTracePoint) Destroy(sequence []TracePoint) {
	for _, value := range sequence {
		FfiDestroyerTracePoint{}.Destroy(value)
	}
}

type FfiConverterMapStringString struct{}

var FfiConverterMapStringStringINSTANCE = FfiConverterMapStringString{}

func (c FfiConverterMapStringString) Lift(rb RustBufferI) map[string]string {
	return LiftFromRustBuffer[map[string]string](c, rb)
}

func (_ FfiConverterMapStringString) Read(reader io.Reader) map[string]string {
	result := make(map[string]string)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterStringINSTANCE.Read(reader)
		value := FfiConverterStringINSTANCE.Read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapStringString) Lower(value map[string]string) C.RustBuffer {
	return LowerIntoRustBuffer[map[string]string](c, value)
}

func (c FfiConverterMapStringString) LowerExternal(value map[string]string) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[map[string]string](c, value))
}

func (_ FfiConverterMapStringString) Write(writer io.Writer, mapValue map[string]string) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]string is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterStringINSTANCE.Write(writer, key)
		FfiConverterStringINSTANCE.Write(writer, value)
	}
}

type FfiDestroyerMapStringString struct{}

func (_ FfiDestroyerMapStringString) Destroy(mapValue map[string]string) {
	for key, value := range mapValue {
		FfiDestroyerString{}.Destroy(key)
		FfiDestroyerString{}.Destroy(value)
	}
}

type FfiConverterMapStringBlobInput struct{}

var FfiConverterMapStringBlobInputINSTANCE = FfiConverterMapStringBlobInput{}

func (c FfiConverterMapStringBlobInput) Lift(rb RustBufferI) map[string]BlobInput {
	return LiftFromRustBuffer[map[string]BlobInput](c, rb)
}

func (_ FfiConverterMapStringBlobInput) Read(reader io.Reader) map[string]BlobInput {
	result := make(map[string]BlobInput)
	length := readInt32(reader)
	for i := int32(0); i < length; i++ {
		key := FfiConverterStringINSTANCE.Read(reader)
		value := FfiConverterBlobInputINSTANCE.Read(reader)
		result[key] = value
	}
	return result
}

func (c FfiConverterMapStringBlobInput) Lower(value map[string]BlobInput) C.RustBuffer {
	return LowerIntoRustBuffer[map[string]BlobInput](c, value)
}

func (c FfiConverterMapStringBlobInput) LowerExternal(value map[string]BlobInput) ExternalCRustBuffer {
	return RustBufferFromC(LowerIntoRustBuffer[map[string]BlobInput](c, value))
}

func (_ FfiConverterMapStringBlobInput) Write(writer io.Writer, mapValue map[string]BlobInput) {
	if len(mapValue) > math.MaxInt32 {
		panic("map[string]BlobInput is too large to fit into Int32")
	}

	writeInt32(writer, int32(len(mapValue)))
	for key, value := range mapValue {
		FfiConverterStringINSTANCE.Write(writer, key)
		FfiConverterBlobInputINSTANCE.Write(writer, value)
	}
}

type FfiDestroyerMapStringBlobInput struct{}

func (_ FfiDestroyerMapStringBlobInput) Destroy(mapValue map[string]BlobInput) {
	for key, value := range mapValue {
		FfiDestroyerString{}.Destroy(key)
		FfiDestroyerBlobInput{}.Destroy(value)
	}
}

// Drop all fonts registered by the host. Existing templates keep theirs.
func ClearFonts() {
	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_oicana_uniffi_fn_func_clear_fonts(_uniffiStatus)
		return false
	})
}

// Configure cache eviction after each compilation.
//
// Keeps cache entries used within the last `max_age` evictions; `None` disables
// automatic eviction. Defaults to 10.
func ConfigureAutomaticCacheEviction(maxAge *uint64) {
	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_oicana_uniffi_fn_func_configure_automatic_cache_eviction(FfiConverterOptionalUint64INSTANCE.Lower(maxAge), _uniffiStatus)
		return false
	})
}

// Set the coloring of compilation diagnostics for all templates.
func ConfigureDiagnosticColor(color DiagnosticColor) {
	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_oicana_uniffi_fn_func_configure_diagnostic_color(FfiConverterDiagnosticColorINSTANCE.Lower(color), _uniffiStatus)
		return false
	})
}

// Evict cache entries not used within the last `max_age` evictions.
func EvictCache(maxAge uint64) {
	rustCall(func(_uniffiStatus *C.RustCallStatus) bool {
		C.uniffi_oicana_uniffi_fn_func_evict_cache(FfiConverterUint64INSTANCE.Lower(maxAge), _uniffiStatus)
		return false
	})
}

// Compile a packed template once and export it, without caching anything.
//
// For repeated compilations of the same template, use [`Template`].
func ExportOnce(files []byte, jsonInputs map[string]string, blobInputs map[string]BlobInput, mode CompilationMode, format ExportFormat, pages *PageRange, limits *ZipLimits) (ExportOnceResult, error) {
	_uniffiRV, _uniffiErr := rustCallWithError[*OicanaError](FfiConverterOicanaError{}, func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_func_export_once(FfiConverterBytesINSTANCE.Lower(files), FfiConverterMapStringStringINSTANCE.Lower(jsonInputs), FfiConverterMapStringBlobInputINSTANCE.Lower(blobInputs), FfiConverterCompilationModeINSTANCE.Lower(mode), FfiConverterExportFormatINSTANCE.Lower(format), FfiConverterOptionalPageRangeINSTANCE.Lower(pages), FfiConverterOptionalZipLimitsINSTANCE.Lower(limits), _uniffiStatus),
		}
	})
	if _uniffiErr != nil {
		var _uniffiDefaultValue ExportOnceResult
		return _uniffiDefaultValue, _uniffiErr
	} else {
		return FfiConverterExportOnceResultINSTANCE.Lift(_uniffiRV), nil
	}
}

// Register fonts from files, or from every font file in a directory tree. Returns the
// number of faces added.
func RegisterFontPaths(paths []string) uint64 {
	return FfiConverterUint64INSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_oicana_uniffi_fn_func_register_font_paths(FfiConverterSequenceStringINSTANCE.Lower(paths), _uniffiStatus)
	}))
}

// Register fonts from their raw file content. Returns the number of faces added.
//
// Templates keep the fonts that were registered when they were created, so register
// fonts first. Nothing deduplicates.
func RegisterFonts(fonts [][]byte) uint64 {
	return FfiConverterUint64INSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) C.uint64_t {
		return C.uniffi_oicana_uniffi_fn_func_register_fonts(FfiConverterSequenceBytesINSTANCE.Lower(fonts), _uniffiStatus)
	}))
}

// All font faces registered by the host.
func RegisteredFonts() []RegisteredFont {
	return FfiConverterSequenceRegisteredFontINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_func_registered_fonts(_uniffiStatus),
		}
	}))
}

// Version of the Oicana library.
func Version() string {
	return FfiConverterStringINSTANCE.Lift(rustCall(func(_uniffiStatus *C.RustCallStatus) RustBufferI {
		return GoRustBuffer{
			inner: C.uniffi_oicana_uniffi_fn_func_version(_uniffiStatus),
		}
	}))
}
