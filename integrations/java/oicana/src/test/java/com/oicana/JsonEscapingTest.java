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
}
