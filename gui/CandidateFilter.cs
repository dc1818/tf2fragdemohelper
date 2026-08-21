using System;
using System.Collections.Generic;
using System.Text;

namespace Tf2StvParserGui
{
    internal sealed class CandidateFilterExpression
    {
        private readonly Dictionary<string, List<string>> positives = new Dictionary<string, List<string>>(StringComparer.OrdinalIgnoreCase);
        private readonly List<CandidateFilterTerm> negatives = new List<CandidateFilterTerm>();
        private readonly List<string> positiveText = new List<string>();
        private readonly List<string> negativeText = new List<string>();

        public static CandidateFilterExpression Parse(string filter)
        {
            CandidateFilterExpression expression = new CandidateFilterExpression();
            List<string> tokens = SplitTokens(filter);
            for (int tokenIndex = 0; tokenIndex < tokens.Count; tokenIndex++)
            {
                string rawToken = tokens[tokenIndex].Trim();
                if (rawToken.Length == 0) continue;
                bool exclude = rawToken[0] == '-';
                bool explicitlyInclude = rawToken[0] == '+';
                string token = exclude || explicitlyInclude ? rawToken.Substring(1) : rawToken;
                if (token.Length == 0) continue;

                string field = "";
                string value = "";
                int separator = FirstSeparator(token);
                if (separator > 0)
                {
                    field = CanonicalField(token.Substring(0, separator));
                    value = Unquote(token.Substring(separator + 1));
                    if (value.Length == 0 && tokenIndex + 1 < tokens.Count && !HasSign(tokens[tokenIndex + 1]))
                    {
                        value = Unquote(tokens[++tokenIndex]);
                    }
                }
                else if ((exclude || explicitlyInclude) && IsField(token))
                {
                    field = CanonicalField(token);
                    if (tokenIndex + 1 < tokens.Count && !HasSign(tokens[tokenIndex + 1]))
                        value = Unquote(tokens[++tokenIndex]);
                }
                else
                {
                    value = Unquote(token);
                }
                if (value.Length == 0) continue;

                if (field.Length == 0)
                {
                    string text = separator > 0 ? Unquote(token) : value;
                    if (exclude) expression.negativeText.Add(text);
                    else expression.positiveText.Add(text);
                    continue;
                }
                if (String.Equals(field, "text", StringComparison.OrdinalIgnoreCase))
                {
                    if (exclude) expression.negativeText.Add(value);
                    else expression.positiveText.Add(value);
                    continue;
                }
                if (exclude)
                {
                    expression.negatives.Add(new CandidateFilterTerm(field, value));
                    continue;
                }
                List<string> values;
                if (!expression.positives.TryGetValue(field, out values))
                {
                    values = new List<string>();
                    expression.positives[field] = values;
                }
                values.Add(value);
            }
            return expression;
        }

        public bool Matches(Func<string, string, bool> fieldMatcher, Func<string, bool> textMatcher)
        {
            if (fieldMatcher == null) throw new ArgumentNullException("fieldMatcher");
            if (textMatcher == null) throw new ArgumentNullException("textMatcher");
            foreach (string value in negativeText)
                if (textMatcher(value)) return false;
            foreach (CandidateFilterTerm term in negatives)
                if (fieldMatcher(term.Field, term.Value)) return false;
            foreach (string value in positiveText)
                if (!textMatcher(value)) return false;
            foreach (KeyValuePair<string, List<string>> pair in positives)
            {
                bool matched = false;
                foreach (string value in pair.Value)
                {
                    if (!fieldMatcher(pair.Key, value)) continue;
                    matched = true;
                    break;
                }
                if (!matched) return false;
            }
            return true;
        }

        private static bool HasSign(string token)
        {
            string value = (token ?? "").Trim();
            return value.Length > 0 && (value[0] == '+' || value[0] == '-');
        }

        private static int FirstSeparator(string token)
        {
            int colon = token.IndexOf(':');
            int equals = token.IndexOf('=');
            if (colon < 0) return equals;
            if (equals < 0) return colon;
            return Math.Min(colon, equals);
        }

        private static string CanonicalField(string field)
        {
            string value = (field ?? "").Trim().ToLowerInvariant();
            if (value == "maps") value = "map";
            else if (value == "classes") value = "class";
            else if (value == "teams") value = "team";
            else if (value == "demos") value = "demo";
            else if (value == "weapons") value = "weapon";
            else if (value == "players") value = "player";
            else if (value == "tags") value = "tag";
            return IsField(value) ? value : "";
        }

        private static bool IsField(string field)
        {
            string value = (field ?? "").Trim().ToLowerInvariant();
            return value == "map" || value == "maps" || value == "class" || value == "classes" ||
                value == "team" || value == "teams" || value == "demo" || value == "demos" ||
                value == "weapon" || value == "weapons" || value == "player" || value == "players" ||
                value == "tag" || value == "tags" || value == "text";
        }

        private static List<string> SplitTokens(string filter)
        {
            List<string> tokens = new List<string>();
            StringBuilder current = new StringBuilder();
            bool inQuotes = false;
            foreach (char character in filter ?? "")
            {
                if (character == '"')
                {
                    inQuotes = !inQuotes;
                    current.Append(character);
                }
                else if ((Char.IsWhiteSpace(character) || character == ',') && !inQuotes)
                {
                    if (current.Length == 0) continue;
                    tokens.Add(current.ToString());
                    current.Length = 0;
                }
                else current.Append(character);
            }
            if (current.Length > 0) tokens.Add(current.ToString());
            return tokens;
        }

        private static string Unquote(string value)
        {
            string text = (value ?? "").Trim();
            if (text.Length >= 2 && text[0] == '"' && text[text.Length - 1] == '"')
                return text.Substring(1, text.Length - 2).Trim();
            return text.Replace("\"", "").Trim();
        }
    }

    internal sealed class CandidateFilterTerm
    {
        public readonly string Field;
        public readonly string Value;

        public CandidateFilterTerm(string field, string value)
        {
            Field = field;
            Value = value;
        }
    }
}
