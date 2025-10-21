using System.Diagnostics.CodeAnalysis;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Oicana.Inputs;

/// <summary>
/// A json input
/// </summary>
public class TemplateJsonInput
{
    /// <summary>
    /// Construct a new input from a key and JSON object
    /// </summary>
    [SetsRequiredMembers]
    public TemplateJsonInput(string key, JsonNode value)
    {
        Key = key;
        Value = value;
    }

    /// <summary>
    /// Construct a new input from a key and JSON object
    /// </summary>
    public static TemplateJsonInput From<T>(string key, T value, JsonSerializerOptions? options = null)
    {
        JsonNode? node = JsonSerializer.SerializeToNode(value, options);
        if (node == null)
        {
            throw new ArgumentException($"Failed to serialize {typeof(T).Name} to JSON node.");
        }

        return new TemplateJsonInput(key, node);
    }

    /// <summary>
    /// key for the json input
    /// </summary>
    /// <example>address</example>
    public required string Key { get; init; }

    /// <summary>
    /// json data as input to compile the template
    /// </summary>
    /// <example>
    /// {
    ///    "test": "from sample data",
    ///    "items": [
    ///        {
    ///            "name": "Frank",
    ///            "one": "first",
    ///            "two": "second",
    ///            "three": "third"
    ///        },
    ///        {
    ///            "name": "John",
    ///            "one": "first_john",
    ///            "two": "second_john",
    ///            "three": "third_john"
    ///        }
    ///    ]
    /// }
    /// </example>
    public required JsonNode Value { get; init; }

    /// <inheritdoc />
    public override bool Equals(object? obj)
    {
        if (obj is TemplateJsonInput input)
        {
            return Key == input.Key && JsonNode.DeepEquals(input.Value, Value);
        }

        return false;
    }

    /// <inheritdoc />
    public override int GetHashCode()
    {
        int hash = (Key?.GetHashCode(StringComparison.Ordinal) ?? 1) * 17;
        hash += (Value?.ToJsonString() ?? string.Empty).GetHashCode(StringComparison.Ordinal) * 23;

        return hash;
    }
}
