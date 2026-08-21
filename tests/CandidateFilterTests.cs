using System;
using System.Collections.Generic;

namespace Tf2StvParserGui
{
    internal static class CandidateFilterTests
    {
        private static int failures;

        private static void Main()
        {
            Dictionary<string, string[]> demomanProduct = Candidate(
                "map", "koth_product_final", "class", "demoman", "team", "red",
                "weapon", "market gardener", "player", "Some Player", "tag", "airshot");
            Dictionary<string, string[]> soldierProcess = Candidate(
                "map", "cp_process_final", "class", "soldier", "team", "blu",
                "weapon", "rocket launcher", "player", "Other Player", "tag", "cleanup");

            Expect(true, Match("+class:demoman", demomanProduct), "single positive");
            Expect(false, Match("-class:demoman", demomanProduct), "single negative");
            Expect(true, Match("+class:demoman +class:soldier", demomanProduct), "same-field OR first");
            Expect(true, Match("+class:demoman +class:soldier", soldierProcess), "same-field OR second");
            Expect(false, Match("+class:demoman +map:cp_process_final", demomanProduct), "cross-field AND");
            Expect(true, Match("+class:demoman +map:koth_product_final", demomanProduct), "positive fields combine");
            Expect(false, Match("-class:spy -map:koth_product_final", demomanProduct), "multiple negatives");
            Expect(true, Match("+class:Demoman -map:tr_rocket_shooting2", demomanProduct), "case normalization");
            Expect(true, Match("+weapon:\"market gardener\"", demomanProduct), "quoted value");
            Expect(true, Match("koth_product_final", demomanProduct), "plain text");
            Expect(true, Match("", demomanProduct), "empty filter");
            Expect(false, Match("unknown:value", demomanProduct), "unknown field is safe text");
            Expect(true, Match("+class:", demomanProduct), "malformed empty value is ignored");
            Expect(false, Match("+map:tr_rocket_shooting2", demomanProduct), "map uses actual map field");

            if (failures != 0) Environment.Exit(1);
            Console.WriteLine("Candidate filter tests passed.");
        }

        private static Dictionary<string, string[]> Candidate(params string[] values)
        {
            Dictionary<string, string[]> result = new Dictionary<string, string[]>(StringComparer.OrdinalIgnoreCase);
            for (int index = 0; index + 1 < values.Length; index += 2)
                result[values[index]] = new string[] { values[index + 1] };
            return result;
        }

        private static bool Match(string filter, Dictionary<string, string[]> candidate)
        {
            CandidateFilterExpression expression = CandidateFilterExpression.Parse(filter);
            return expression.Matches(
                delegate(string field, string value)
                {
                    string[] actual;
                    if (!candidate.TryGetValue(field, out actual)) return false;
                    foreach (string item in actual)
                        if (item.IndexOf(value, StringComparison.OrdinalIgnoreCase) >= 0) return true;
                    return false;
                },
                delegate(string value)
                {
                    foreach (string[] actual in candidate.Values)
                        foreach (string item in actual)
                            if (item.IndexOf(value, StringComparison.OrdinalIgnoreCase) >= 0) return true;
                    return false;
                });
        }

        private static void Expect(bool expected, bool actual, string name)
        {
            if (expected == actual) return;
            failures++;
            Console.Error.WriteLine("FAILED: " + name + " expected " + expected + " but got " + actual);
        }
    }
}
