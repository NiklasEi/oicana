package com.oicana;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for the JSON escaping used when serializing blob metadata.
 */
class JsonEscapingTest {

    @Test
    void escapesQuotesAndBackslashes() {
        assertEquals("a \\\"quoted\\\" \\\\path\\\\", Template.escapeJson("a \"quoted\" \\path\\"));
    }

    @Test
    void escapesNamedControlCharacters() {
        assertEquals("\\n\\r\\t\\b\\f", Template.escapeJson("\n\r\t\b\f"));
    }

    @Test
    void escapesRemainingControlCharacters() {
        String input = "\0" + (char) 0x01 + (char) 0x1F;
        assertEquals("\\u0000\\u0001\\u001f", Template.escapeJson(input));
    }

    @Test
    void leavesPlainTextAndNonAsciiUntouched() {
        assertEquals("plain text 123", Template.escapeJson("plain text 123"));
        assertEquals(" ~é世", Template.escapeJson(" ~é世"));
    }

    @Test
    void rejectsNonFiniteNumbers() {
        assertThrows(IllegalArgumentException.class, () -> Template.valueToJson(Double.NaN));
        assertThrows(IllegalArgumentException.class,
                () -> Template.valueToJson(Double.POSITIVE_INFINITY));
        assertThrows(IllegalArgumentException.class,
                () -> Template.valueToJson(Float.NEGATIVE_INFINITY));
    }

    @Test
    void serializesPrimitiveArrays() {
        assertEquals("[1,2,3]", Template.valueToJson(new int[] {1, 2, 3}));
        assertEquals("[1.5,2.5]", Template.valueToJson(new double[] {1.5, 2.5}));
        assertEquals("[true,false]", Template.valueToJson(new boolean[] {true, false}));
    }

    @Test
    void serializesObjectArraysAndNestedValues() {
        assertEquals("[\"a\",1,null]", Template.valueToJson(new Object[] {"a", 1, null}));
        assertEquals("[[1,2],[3]]", Template.valueToJson(new int[][] {{1, 2}, {3}}));
    }
}
