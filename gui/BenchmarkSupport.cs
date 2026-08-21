using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Web.Script.Serialization;

namespace Tf2StvParserGui
{
    internal sealed class WorkerRunResult
    {
        public DateTime StartedUtc;
        public DateTime FinishedUtc;
        public double WallSeconds;
        public double CpuSeconds;
        public long PeakWorkingSetBytes;
        public int ExitCode;
        public string Executable;
    }

    internal sealed class EtaSnapshot
    {
        public int Phase;
        public int Completed;
        public int Total;
        public double Fraction;
        public double ElapsedSeconds;
        public double RemainingSeconds;
        public DateTime EstimatedCompletionLocal;
        public bool IsEstimateAvailable;

        public string ShortText()
        {
            if (!IsEstimateAvailable) return "ETA: calibrating...";
            return "ETA: " + BenchmarkFormatting.Duration(RemainingSeconds) +
                " (about " + EstimatedCompletionLocal.ToString("h:mm:ss tt") + ")";
        }
    }

    internal sealed class PhaseEtaTracker
    {
        private readonly object sync = new object();
        private readonly int phase;
        private readonly int total;
        private readonly double totalWeight;
        private readonly double historicalSecondsPerGiB;
        private readonly int workerCount;
        private readonly Stopwatch watch = new Stopwatch();
        private int completed;
        private double completedWeight;

        public PhaseEtaTracker(int phase, IList<double> weights, double historicalSecondsPerGiB, int workerCount)
        {
            this.phase = phase;
            total = weights == null ? 0 : weights.Count;
            this.historicalSecondsPerGiB = historicalSecondsPerGiB;
            this.workerCount = Math.Max(1, workerCount);
            double sum = 0.0;
            if (weights != null)
            {
                foreach (double weight in weights) sum += Math.Max(1.0, weight);
            }
            totalWeight = Math.Max(1.0, sum);
        }

        public void Start()
        {
            lock (sync)
            {
                completed = 0;
                completedWeight = 0.0;
                watch.Restart();
            }
        }

        public EtaSnapshot InitialSnapshot()
        {
            lock (sync) return BuildSnapshot();
        }

        public EtaSnapshot Complete(double weight)
        {
            lock (sync)
            {
                completed++;
                completedWeight += Math.Max(1.0, weight);
                if (completedWeight > totalWeight) completedWeight = totalWeight;
                return BuildSnapshot();
            }
        }

        private EtaSnapshot BuildSnapshot()
        {
            EtaSnapshot snapshot = new EtaSnapshot();
            snapshot.Phase = phase;
            snapshot.Completed = Math.Min(completed, total);
            snapshot.Total = total;
            snapshot.ElapsedSeconds = watch.Elapsed.TotalSeconds;
            snapshot.Fraction = Math.Max(0.0, Math.Min(1.0, completedWeight / totalWeight));

            double remaining = -1.0;
            if (snapshot.Fraction > 0.01 && snapshot.ElapsedSeconds >= 2.0)
            {
                double weightedRate = completedWeight / snapshot.ElapsedSeconds;
                if (weightedRate > 0.0)
                    remaining = Math.Max(0.0, (totalWeight - completedWeight) / weightedRate);
            }
            else if (historicalSecondsPerGiB > 0.0)
            {
                double totalGiB = totalWeight / BenchmarkFormatting.GiB;
                remaining = Math.Max(0.0, (totalGiB * historicalSecondsPerGiB) / workerCount);
            }

            if (remaining >= 0.0 && !Double.IsNaN(remaining) && !Double.IsInfinity(remaining))
            {
                snapshot.IsEstimateAvailable = true;
                snapshot.RemainingSeconds = remaining;
                snapshot.EstimatedCompletionLocal = DateTime.Now.AddSeconds(remaining);
            }
            return snapshot;
        }
    }

    internal sealed class DiskPreflightEstimate
    {
        public readonly string DriveRoot;
        public readonly ulong AvailableFreeBytes;
        public readonly ulong TotalInputBytes;
        public readonly ulong EstimatedParseOutputBytes;
        public readonly ulong EstimatedAnalysisAddedBytes;
        public readonly ulong EstimatedOutputBytes;
        public readonly ulong SafetyHeadroomBytes;
        public readonly ulong SafeRequiredFreeBytes;
        public readonly double ParseExpansionRatio;
        public readonly double AnalysisExpansionRatio;
        public readonly int HistoricalParseSamples;
        public readonly int HistoricalAnalysisSamples;

        public DiskPreflightEstimate(string driveRoot, ulong availableFreeBytes, ulong totalInputBytes,
            ulong estimatedParseOutputBytes, ulong estimatedAnalysisAddedBytes, ulong safetyHeadroomBytes,
            double parseExpansionRatio, double analysisExpansionRatio,
            int historicalParseSamples, int historicalAnalysisSamples)
        {
            DriveRoot = driveRoot;
            AvailableFreeBytes = availableFreeBytes;
            TotalInputBytes = totalInputBytes;
            EstimatedParseOutputBytes = estimatedParseOutputBytes;
            EstimatedAnalysisAddedBytes = estimatedAnalysisAddedBytes;
            EstimatedOutputBytes = SafeAdd(estimatedParseOutputBytes, estimatedAnalysisAddedBytes);
            SafetyHeadroomBytes = safetyHeadroomBytes;
            SafeRequiredFreeBytes = SafeAdd(EstimatedOutputBytes, safetyHeadroomBytes);
            ParseExpansionRatio = parseExpansionRatio;
            AnalysisExpansionRatio = analysisExpansionRatio;
            HistoricalParseSamples = historicalParseSamples;
            HistoricalAnalysisSamples = historicalAnalysisSamples;
        }

        public bool HasEstimatedOutputSpace
        {
            get { return AvailableFreeBytes == 0 || AvailableFreeBytes >= EstimatedOutputBytes; }
        }

        public bool HasSafeSpace
        {
            get { return AvailableFreeBytes == 0 || AvailableFreeBytes >= SafeRequiredFreeBytes; }
        }

        public ulong EstimateParseForDemo(string demoPath)
        {
            ulong input = BenchmarkFormatting.FileSize(demoPath);
            double estimated = Math.Max(64.0 * BenchmarkFormatting.MiB, input * ParseExpansionRatio);
            return BenchmarkFormatting.ToUInt64Saturated(estimated);
        }

        public ulong EstimateAnalysisForDemo(string demoPath)
        {
            ulong input = BenchmarkFormatting.FileSize(demoPath);
            double estimated = Math.Max(8.0 * BenchmarkFormatting.MiB, input * AnalysisExpansionRatio);
            return BenchmarkFormatting.ToUInt64Saturated(estimated);
        }

        public string Describe()
        {
            StringBuilder text = new StringBuilder();
            text.AppendLine("DISK PREFLIGHT ESTIMATE");
            text.AppendLine("Output drive: " + (String.IsNullOrWhiteSpace(DriveRoot) ? "unknown" : DriveRoot));
            text.AppendLine("Input demos: " + BenchmarkFormatting.Bytes(TotalInputBytes));
            text.AppendLine("Estimated full parse output: " + BenchmarkFormatting.Bytes(EstimatedParseOutputBytes) +
                " (" + ParseExpansionRatio.ToString("0.00") + "x source size)");
            text.AppendLine("Estimated candidate-analysis additions: " + BenchmarkFormatting.Bytes(EstimatedAnalysisAddedBytes) +
                " (" + AnalysisExpansionRatio.ToString("0.00") + "x source size)");
            text.AppendLine("Estimated output total: " + BenchmarkFormatting.Bytes(EstimatedOutputBytes));
            text.AppendLine("Safety headroom: " + BenchmarkFormatting.Bytes(SafetyHeadroomBytes));
            text.AppendLine("Recommended free space before starting: " + BenchmarkFormatting.Bytes(SafeRequiredFreeBytes));
            if (AvailableFreeBytes > 0)
                text.AppendLine("Currently free: " + BenchmarkFormatting.Bytes(AvailableFreeBytes));
            else
                text.AppendLine("Currently free: unavailable");
            text.AppendLine("Historical samples used: parse=" + HistoricalParseSamples + ", analysis=" + HistoricalAnalysisSamples + ".");
            if (HistoricalParseSamples < 3)
                text.AppendLine("Parse output ratio is using a conservative cold-start estimate; it will learn from completed batches.");
            return text.ToString();
        }

        private static ulong SafeAdd(ulong a, ulong b)
        {
            if (UInt64.MaxValue - a < b) return UInt64.MaxValue;
            return a + b;
        }
    }

    internal sealed class BenchmarkHistory
    {
        private const int MaxLoadedLines = 5000;
        private readonly List<IDictionary<string, object>> records = new List<IDictionary<string, object>>();
        private readonly string historyPath;
        private readonly object writeSync = new object();

        private BenchmarkHistory(string path)
        {
            historyPath = path;
        }

        public string HistoryPath { get { return historyPath; } }

        public static BenchmarkHistory Load()
        {
            string folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "TF2FragDemoHelper");
            string path = Path.Combine(folder, "benchmark_history.ndjson");
            BenchmarkHistory history = new BenchmarkHistory(path);
            try
            {
                if (!File.Exists(path)) return history;
                JavaScriptSerializer serializer = NewSerializer();
                Queue<string> lines = new Queue<string>();
                foreach (string line in File.ReadLines(path))
                {
                    if (String.IsNullOrWhiteSpace(line)) continue;
                    lines.Enqueue(line);
                    if (lines.Count > MaxLoadedLines) lines.Dequeue();
                }
                foreach (string line in lines)
                {
                    try
                    {
                        IDictionary<string, object> record = serializer.Deserialize<Dictionary<string, object>>(line);
                        if (record != null) history.records.Add(record);
                    }
                    catch { }
                }
            }
            catch { }
            return history;
        }

        public double ParseExpansionRatio(out int samples)
        {
            List<double> values = Values("parse", "output_ratio", 0.1, 100.0, false);
            samples = values.Count;
            if (values.Count < 3) return 16.0;
            // 90th percentile plus 10% protects against unusually verbose demos.
            return Clamp(Percentile(values, 0.90) * 1.10, 4.0, 50.0);
        }

        public double AnalysisExpansionRatio(out int samples)
        {
            List<double> values = Values("analysis", "added_output_ratio", 0.0, 10.0, false);
            samples = values.Count;
            if (values.Count < 3) return 0.25;
            return Clamp(Percentile(values, 0.90) * 1.20, 0.02, 3.0);
        }

        public ulong HistoricalParsePeakBytes(ulong fallback)
        {
            List<double> values = Values("parse", "peak_working_set_bytes", 64.0 * BenchmarkFormatting.MiB, 128.0 * BenchmarkFormatting.GiB, true);
            if (values.Count < 5) return fallback;
            ulong historical = BenchmarkFormatting.ToUInt64Saturated(Percentile(values, 0.90) * 1.20);
            ulong minimum = 512UL * 1024UL * 1024UL;
            return historical > minimum ? historical : minimum;
        }

        public ulong HistoricalAnalysisPeakBytes(ulong fallback)
        {
            List<double> values = Values("analysis", "peak_working_set_bytes", 64.0 * BenchmarkFormatting.MiB, 128.0 * BenchmarkFormatting.GiB, true);
            if (values.Count < 5) return fallback;
            ulong historical = BenchmarkFormatting.ToUInt64Saturated(Percentile(values, 0.90) * 1.20);
            ulong minimum = 512UL * 1024UL * 1024UL;
            return historical > minimum ? historical : minimum;
        }

        public double ParseSecondsPerGiB()
        {
            return SecondsPerGiB("parse", "input_bytes", "wall_seconds");
        }

        public double AnalysisSecondsPerGiB()
        {
            return SecondsPerGiB("analysis", "analysis_input_bytes", "wall_seconds");
        }

        public int RecommendParseWorkers(int currentMaximum)
        {
            return RecommendWorkers("parse_workers", "parse_source_mib_per_sec_wall", currentMaximum);
        }

        public int RecommendAnalysisWorkers(int currentMaximum)
        {
            return RecommendWorkers("analysis_workers", "analysis_input_mib_per_sec_wall", currentMaximum);
        }

        private int RecommendWorkers(string workerKey, string throughputKey, int currentMaximum)
        {
            if (currentMaximum <= 1) return Math.Max(1, currentMaximum);
            Dictionary<int, List<double>> byWorkers = new Dictionary<int, List<double>>();
            int successful = 0;
            foreach (IDictionary<string, object> record in records)
            {
                if (!String.Equals(StringValue(record, "kind"), "batch", StringComparison.OrdinalIgnoreCase)) continue;
                if (!String.Equals(StringValue(record, "status"), "success", StringComparison.OrdinalIgnoreCase)) continue;
                if (!IsSameMachine(record)) continue;
                int workers = (int)DoubleValue(record, workerKey, 0.0);
                double throughput = DoubleValue(record, throughputKey, 0.0);
                if (workers < 1 || workers > currentMaximum || throughput <= 0.0) continue;
                List<double> values;
                if (!byWorkers.TryGetValue(workers, out values))
                {
                    values = new List<double>();
                    byWorkers[workers] = values;
                }
                values.Add(throughput);
                successful++;
            }

            // Avoid tuning from one noisy run. We need several successful runs and at least
            // two distinct worker counts before changing the hardware-derived ceiling.
            if (successful < 4 || byWorkers.Count < 2) return currentMaximum;

            int bestWorkers = currentMaximum;
            double bestThroughput = -1.0;
            foreach (KeyValuePair<int, List<double>> pair in byWorkers)
            {
                if (pair.Value.Count < 2) continue;
                double median = Percentile(new List<double>(pair.Value), 0.50);
                if (median > bestThroughput)
                {
                    bestThroughput = median;
                    bestWorkers = pair.Key;
                }
            }
            return bestThroughput > 0.0 ? Math.Max(1, Math.Min(currentMaximum, bestWorkers)) : currentMaximum;
        }

        public DiskPreflightEstimate EstimateDisk(IList<BatchExportEntry> entries, string exportRoot)
        {
            int parseSamples;
            int analysisSamples;
            double parseRatio = ParseExpansionRatio(out parseSamples);
            double analysisRatio = AnalysisExpansionRatio(out analysisSamples);
            ulong totalInput = 0;
            ulong parseBytes = 0;
            ulong analysisBytes = 0;
            if (entries != null)
            {
                foreach (BatchExportEntry entry in entries)
                {
                    ulong input = BenchmarkFormatting.FileSize(entry.DemoPath);
                    totalInput = SafeAdd(totalInput, input);
                    parseBytes = SafeAdd(parseBytes,
                        BenchmarkFormatting.ToUInt64Saturated(Math.Max(64.0 * BenchmarkFormatting.MiB, input * parseRatio)));
                    analysisBytes = SafeAdd(analysisBytes,
                        BenchmarkFormatting.ToUInt64Saturated(Math.Max(8.0 * BenchmarkFormatting.MiB, input * analysisRatio)));
                }
            }
            ulong estimated = SafeAdd(parseBytes, analysisBytes);
            ulong proportionalHeadroom = BenchmarkFormatting.ToUInt64Saturated(estimated * 0.20);
            ulong minimumHeadroom = 8UL * 1024UL * 1024UL * 1024UL;
            ulong headroom = proportionalHeadroom > minimumHeadroom ? proportionalHeadroom : minimumHeadroom;
            string driveRoot;
            ulong free = SystemDiskInfo.AvailableFreeBytes(exportRoot, out driveRoot);
            return new DiskPreflightEstimate(driveRoot, free, totalInput, parseBytes, analysisBytes, headroom,
                parseRatio, analysisRatio, parseSamples, analysisSamples);
        }

        public void AppendJobRecord(IDictionary<string, object> record)
        {
            if (record == null) return;
            lock (writeSync)
            {
                try
                {
                    string folder = Path.GetDirectoryName(historyPath);
                    if (!Directory.Exists(folder)) Directory.CreateDirectory(folder);
                    JavaScriptSerializer serializer = NewSerializer();
                    using (StreamWriter writer = new StreamWriter(historyPath, true, new UTF8Encoding(false)))
                        writer.WriteLine(serializer.Serialize(record));
                    records.Add(new Dictionary<string, object>(record));
                }
                catch { }
            }
        }

        private List<double> Values(string kind, string key, double min, double max, bool sameMachineOnly)
        {
            List<double> values = new List<double>();
            foreach (IDictionary<string, object> record in records)
            {
                if (!String.Equals(StringValue(record, "kind"), kind, StringComparison.OrdinalIgnoreCase)) continue;
                if (sameMachineOnly && !IsSameMachine(record)) continue;
                double value = DoubleValue(record, key, -1.0);
                if (value >= min && value <= max) values.Add(value);
            }
            return values;
        }

        private double SecondsPerGiB(string kind, string bytesKey, string secondsKey)
        {
            List<double> rates = new List<double>();
            foreach (IDictionary<string, object> record in records)
            {
                if (!String.Equals(StringValue(record, "kind"), kind, StringComparison.OrdinalIgnoreCase)) continue;
                if (!IsSameMachine(record)) continue;
                double bytes = DoubleValue(record, bytesKey, 0.0);
                double seconds = DoubleValue(record, secondsKey, 0.0);
                if (bytes < 16.0 * BenchmarkFormatting.MiB || seconds <= 0.0) continue;
                double gib = bytes / BenchmarkFormatting.GiB;
                if (gib > 0.0) rates.Add(seconds / gib);
            }
            if (rates.Count < 3) return -1.0;
            return Percentile(rates, 0.50);
        }

        private static bool IsSameMachine(IDictionary<string, object> record)
        {
            if (record == null) return false;
            int logical = (int)DoubleValue(record, "logical_processors", 0.0);
            if (logical > 0 && logical != Environment.ProcessorCount) return false;
            string recordedCpu = StringValue(record, "processor_identifier");
            string currentCpu = Environment.GetEnvironmentVariable("PROCESSOR_IDENTIFIER") ?? "";
            if (!String.IsNullOrWhiteSpace(recordedCpu) && !String.IsNullOrWhiteSpace(currentCpu) &&
                !String.Equals(recordedCpu, currentCpu, StringComparison.OrdinalIgnoreCase)) return false;
            return true;
        }

        private static string StringValue(IDictionary<string, object> dict, string key)
        {
            object value;
            return dict != null && dict.TryGetValue(key, out value) && value != null ? Convert.ToString(value) : "";
        }

        private static double DoubleValue(IDictionary<string, object> dict, string key, double fallback)
        {
            object value;
            if (dict == null || !dict.TryGetValue(key, out value) || value == null) return fallback;
            try { return Convert.ToDouble(value, CultureInfo.InvariantCulture); }
            catch { return fallback; }
        }

        private static double Percentile(List<double> values, double percentile)
        {
            if (values == null || values.Count == 0) return 0.0;
            values.Sort();
            int index = (int)Math.Ceiling((values.Count - 1) * percentile);
            if (index < 0) index = 0;
            if (index >= values.Count) index = values.Count - 1;
            return values[index];
        }

        private static double Clamp(double value, double minimum, double maximum)
        {
            if (value < minimum) return minimum;
            if (value > maximum) return maximum;
            return value;
        }

        private static ulong SafeAdd(ulong a, ulong b)
        {
            if (UInt64.MaxValue - a < b) return UInt64.MaxValue;
            return a + b;
        }

        private static JavaScriptSerializer NewSerializer()
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            serializer.MaxJsonLength = Int32.MaxValue;
            serializer.RecursionLimit = 256;
            return serializer;
        }
    }

    internal sealed class BenchmarkSession : IDisposable
    {
        private readonly object sync = new object();
        private readonly string benchmarkDirectory;
        private readonly string parseCsv;
        private readonly string analysisCsv;
        private readonly string resourceCsv;
        private readonly string etaCsv;
        private readonly string failuresCsv;
        private readonly string summaryJson;
        private readonly BenchmarkHistory history;
        private readonly string exportRoot;
        private readonly DateTime startedUtc = DateTime.UtcNow;
        private readonly Stopwatch totalWatch = Stopwatch.StartNew();
        private readonly Dictionary<int, ulong> parseOutputBytesByOrder = new Dictionary<int, ulong>();
        private readonly List<double> observedParseRatios = new List<double>();
        private readonly List<double> observedAnalysisRatios = new List<double>();
        private int currentPhase;
        private int currentCompleted;
        private int currentTotal;
        private int currentWorkerLimit;
        private double phase1Seconds;
        private double phase2Seconds;
        private int finalCandidateCount;
        private string finalStatus = "running";
        private ulong totalParseOutputBytes;
        private ulong totalAnalysisInputBytes;
        private ulong totalAnalysisAddedBytes;
        private double totalParseProcessCpuSeconds;
        private double totalAnalysisProcessCpuSeconds;
        private long maxParsePeakWorkingSetBytes;
        private long maxAnalysisPeakWorkingSetBytes;
        private int successfulParseJobs;
        private int successfulAnalysisJobs;
        private double sampledCpuSum;
        private int sampledCpuCount;
        private ulong minimumAvailableRamBytes = UInt64.MaxValue;
        private ulong minimumFreeDiskBytes = UInt64.MaxValue;

        public BenchmarkSession(string exportRoot, IList<BatchExportEntry> entries, ResourcePlan plan,
            DiskPreflightEstimate preflight, BenchmarkHistory history)
        {
            this.exportRoot = exportRoot;
            this.history = history;
            benchmarkDirectory = Path.Combine(exportRoot, "benchmark");
            Directory.CreateDirectory(benchmarkDirectory);
            parseCsv = Path.Combine(benchmarkDirectory, "parse_metrics.csv");
            analysisCsv = Path.Combine(benchmarkDirectory, "analysis_metrics.csv");
            resourceCsv = Path.Combine(benchmarkDirectory, "resource_samples.csv");
            etaCsv = Path.Combine(benchmarkDirectory, "eta_samples.csv");
            failuresCsv = Path.Combine(benchmarkDirectory, "failures.csv");
            summaryJson = Path.Combine(benchmarkDirectory, "benchmark_summary.json");

            WriteHeader(parseCsv, "timestamp_utc,order,demo,input_bytes,parse_output_bytes,output_ratio,wall_seconds,process_cpu_seconds,peak_working_set_bytes,input_mib_per_sec,output_mib_per_sec,worker_limit");
            WriteHeader(analysisCsv, "timestamp_utc,order,demo,analysis_input_bytes,analysis_added_output_bytes,added_output_ratio,wall_seconds,process_cpu_seconds,peak_working_set_bytes,input_mib_per_sec,candidate_count,worker_limit,capture_type,analysis_scope,total_player_death_events,accepted_live_scope_kills,rejected_outside_live_round,rejected_not_pov_attacker,state_lookup_count,unindexed_state_lookup_count,projectile_tracks_total,projectile_tracks_examined,candidate_group_jobs,candidate_workers_used,state_enrichment_seconds,candidate_scoring_seconds,analyzer_total_seconds");
            WriteHeader(resourceCsv, "timestamp_utc,elapsed_seconds,phase,completed,total,worker_limit,active_processes,cpu_percent,available_ram_bytes,free_disk_bytes");
            WriteHeader(etaCsv, "timestamp_utc,phase,completed,total,fraction,elapsed_seconds,remaining_seconds,estimated_completion_local");
            WriteHeader(failuresCsv, "timestamp_utc,phase,order,demo,message");

            Dictionary<string, object> initial = BuildBaseSummary(entries, plan, preflight);
            initial["status"] = "running";
            initial["benchmark_directory"] = benchmarkDirectory;
            initial["persistent_history"] = history == null ? "" : history.HistoryPath;
            WriteJson(summaryJson, initial);
            try
            {
                if (history != null && File.Exists(history.HistoryPath))
                    File.Copy(history.HistoryPath, Path.Combine(benchmarkDirectory, "benchmark_history_before_run.ndjson"), true);
            }
            catch { }
        }

        public string DirectoryPath { get { return benchmarkDirectory; } }
        public int CurrentPhase { get { lock (sync) return currentPhase; } }
        public int CurrentCompleted { get { lock (sync) return currentCompleted; } }
        public int CurrentTotal { get { lock (sync) return currentTotal; } }
        public int CurrentWorkerLimit { get { lock (sync) return currentWorkerLimit; } }

        public void SetPhase(int phase, int total, int workerLimit)
        {
            lock (sync)
            {
                currentPhase = phase;
                currentCompleted = 0;
                currentTotal = total;
                currentWorkerLimit = workerLimit;
            }
        }

        public void SetPhaseCompleted(int completed)
        {
            lock (sync) currentCompleted = completed;
        }

        public ulong ParseOutputBytes(int order)
        {
            lock (sync)
            {
                ulong value;
                return parseOutputBytesByOrder.TryGetValue(order, out value) ? value : 0;
            }
        }

        public ulong EstimateParseWriteBytes(BatchExportEntry entry, DiskPreflightEstimate preflight)
        {
            ulong input = BenchmarkFormatting.FileSize(entry.DemoPath);
            double ratio = preflight == null ? 16.0 : preflight.ParseExpansionRatio;
            lock (sync)
            {
                foreach (double observed in observedParseRatios)
                    if (observed * 1.15 > ratio) ratio = observed * 1.15;
            }
            return BenchmarkFormatting.ToUInt64Saturated(Math.Max(64.0 * BenchmarkFormatting.MiB, input * ratio));
        }

        public ulong EstimateAnalysisWriteBytes(BatchExportEntry entry, DiskPreflightEstimate preflight)
        {
            ulong input = BenchmarkFormatting.FileSize(entry.DemoPath);
            double ratio = preflight == null ? 0.25 : preflight.AnalysisExpansionRatio;
            lock (sync)
            {
                foreach (double observed in observedAnalysisRatios)
                    if (observed * 1.20 > ratio) ratio = observed * 1.20;
            }
            return BenchmarkFormatting.ToUInt64Saturated(Math.Max(8.0 * BenchmarkFormatting.MiB, input * ratio));
        }

        public void RecordParse(BatchExportEntry entry, WorkerRunResult result, ulong outputBytes, int workerLimit)
        {
            ulong inputBytes = BenchmarkFormatting.FileSize(entry.DemoPath);
            double ratio = inputBytes > 0 ? outputBytes / (double)inputBytes : 0.0;
            double inputRate = result.WallSeconds > 0.0 ? (inputBytes / (double)BenchmarkFormatting.MiB) / result.WallSeconds : 0.0;
            double outputRate = result.WallSeconds > 0.0 ? (outputBytes / (double)BenchmarkFormatting.MiB) / result.WallSeconds : 0.0;
            lock (sync)
            {
                parseOutputBytesByOrder[entry.Order] = outputBytes;
                if (ratio > 0.0) observedParseRatios.Add(ratio);
                totalParseOutputBytes = SafeAdd(totalParseOutputBytes, outputBytes);
                totalParseProcessCpuSeconds += result.CpuSeconds;
                if (result.PeakWorkingSetBytes > maxParsePeakWorkingSetBytes) maxParsePeakWorkingSetBytes = result.PeakWorkingSetBytes;
                successfulParseJobs++;
                AppendCsv(parseCsv,
                    Csv(DateTime.UtcNow.ToString("o")), entry.Order.ToString(), Csv(Path.GetFileName(entry.DemoPath)),
                    inputBytes.ToString(), outputBytes.ToString(), ratio.ToString("0.0000", CultureInfo.InvariantCulture),
                    result.WallSeconds.ToString("0.000", CultureInfo.InvariantCulture),
                    result.CpuSeconds.ToString("0.000", CultureInfo.InvariantCulture), result.PeakWorkingSetBytes.ToString(),
                    inputRate.ToString("0.000", CultureInfo.InvariantCulture), outputRate.ToString("0.000", CultureInfo.InvariantCulture),
                    workerLimit.ToString());
            }

            if (history != null)
            {
                Dictionary<string, object> record = CommonHistoryRecord("parse", entry, result, workerLimit);
                record["input_bytes"] = inputBytes;
                record["output_bytes"] = outputBytes;
                record["output_ratio"] = ratio;
                history.AppendJobRecord(record);
            }
        }

        public void RecordAnalysis(BatchExportEntry entry, WorkerRunResult result, ulong totalOutputBytesAfter,
            int candidateCount, int workerLimit)
        {
            ulong parseBytes = ParseOutputBytes(entry.Order);
            if (parseBytes == 0) parseBytes = BenchmarkFormatting.DirectorySize(entry.ExportDirectory);
            ulong added = totalOutputBytesAfter > parseBytes ? totalOutputBytesAfter - parseBytes : 0;
            ulong sourceDemo = BenchmarkFormatting.FileSize(entry.DemoPath);
            double ratio = sourceDemo > 0 ? added / (double)sourceDemo : 0.0;
            double inputRate = result.WallSeconds > 0.0 ? (parseBytes / (double)BenchmarkFormatting.MiB) / result.WallSeconds : 0.0;
            Dictionary<string, object> analyzerProfile = ReadJsonDictionary(Path.Combine(entry.ExportDirectory, "analysis_profile.json"));
            IDictionary<string, object> deathRejections = DictionaryValue(analyzerProfile, "death_rejections");
            IDictionary<string, object> stageSeconds = DictionaryValue(analyzerProfile, "stage_seconds");
            string captureType = TextValue(analyzerProfile, "capture_type");
            string analysisScope = TextValue(analyzerProfile, "analysis_scope");
            long totalPlayerDeaths = LongValue(analyzerProfile, "total_player_death_events");
            long acceptedKills = LongValue(analyzerProfile, "accepted_live_scope_kills");
            long outsideRound = LongValue(deathRejections, "outside_live_round");
            long notPov = LongValue(deathRejections, "not_pov_attacker");
            long stateLookups = LongValue(analyzerProfile, "state_lookup_count");
            long unindexedLookups = LongValue(analyzerProfile, "unindexed_state_lookup_count");
            long projectileTracks = LongValue(analyzerProfile, "projectile_tracks_total");
            long projectileExamined = LongValue(analyzerProfile, "projectile_tracks_examined");
            long candidateGroupJobs = LongValue(analyzerProfile, "candidate_group_jobs");
            long candidateWorkersUsed = LongValue(analyzerProfile, "candidate_workers_used");
            double stateEnrichmentSeconds = DoubleValue(stageSeconds, "state_enrichment");
            double candidateScoringSeconds = DoubleValue(stageSeconds, "candidate_grouping_and_scoring");
            double analyzerTotalSeconds = DoubleValue(analyzerProfile, "total_seconds");
            lock (sync)
            {
                if (ratio >= 0.0) observedAnalysisRatios.Add(ratio);
                totalAnalysisInputBytes = SafeAdd(totalAnalysisInputBytes, parseBytes);
                totalAnalysisAddedBytes = SafeAdd(totalAnalysisAddedBytes, added);
                totalAnalysisProcessCpuSeconds += result.CpuSeconds;
                if (result.PeakWorkingSetBytes > maxAnalysisPeakWorkingSetBytes) maxAnalysisPeakWorkingSetBytes = result.PeakWorkingSetBytes;
                successfulAnalysisJobs++;
                AppendCsv(analysisCsv,
                    Csv(DateTime.UtcNow.ToString("o")), entry.Order.ToString(), Csv(Path.GetFileName(entry.DemoPath)),
                    parseBytes.ToString(), added.ToString(), ratio.ToString("0.0000", CultureInfo.InvariantCulture),
                    result.WallSeconds.ToString("0.000", CultureInfo.InvariantCulture),
                    result.CpuSeconds.ToString("0.000", CultureInfo.InvariantCulture), result.PeakWorkingSetBytes.ToString(),
                    inputRate.ToString("0.000", CultureInfo.InvariantCulture), candidateCount.ToString(), workerLimit.ToString(),
                    Csv(captureType), Csv(analysisScope), totalPlayerDeaths.ToString(), acceptedKills.ToString(),
                    outsideRound.ToString(), notPov.ToString(), stateLookups.ToString(), unindexedLookups.ToString(),
                    projectileTracks.ToString(), projectileExamined.ToString(), candidateGroupJobs.ToString(), candidateWorkersUsed.ToString(),
                    stateEnrichmentSeconds.ToString("0.000000", CultureInfo.InvariantCulture),
                    candidateScoringSeconds.ToString("0.000000", CultureInfo.InvariantCulture),
                    analyzerTotalSeconds.ToString("0.000000", CultureInfo.InvariantCulture));
            }

            if (history != null)
            {
                Dictionary<string, object> record = CommonHistoryRecord("analysis", entry, result, workerLimit);
                record["input_bytes"] = sourceDemo;
                record["analysis_input_bytes"] = parseBytes;
                record["added_output_bytes"] = added;
                record["added_output_ratio"] = ratio;
                record["candidate_count"] = candidateCount;
                record["capture_type"] = captureType;
                record["analysis_scope"] = analysisScope;
                record["total_player_death_events"] = totalPlayerDeaths;
                record["accepted_live_scope_kills"] = acceptedKills;
                record["rejected_outside_live_round"] = outsideRound;
                record["rejected_not_pov_attacker"] = notPov;
                record["state_lookup_count"] = stateLookups;
                record["projectile_tracks_total"] = projectileTracks;
                record["projectile_tracks_examined"] = projectileExamined;
                record["candidate_group_jobs"] = candidateGroupJobs;
                record["candidate_workers_used"] = candidateWorkersUsed;
                record["analyzer_total_seconds"] = analyzerTotalSeconds;
                history.AppendJobRecord(record);
            }
        }

        public void RecordEta(EtaSnapshot eta)
        {
            if (eta == null) return;
            lock (sync)
            {
                AppendCsv(etaCsv,
                    Csv(DateTime.UtcNow.ToString("o")), eta.Phase.ToString(), eta.Completed.ToString(), eta.Total.ToString(),
                    eta.Fraction.ToString("0.000000", CultureInfo.InvariantCulture),
                    eta.ElapsedSeconds.ToString("0.000", CultureInfo.InvariantCulture),
                    eta.IsEstimateAvailable ? eta.RemainingSeconds.ToString("0.000", CultureInfo.InvariantCulture) : "",
                    eta.IsEstimateAvailable ? Csv(eta.EstimatedCompletionLocal.ToString("o")) : "");
            }
        }

        public void RecordResourceSample(int activeProcesses)
        {
            string driveRoot;
            ulong freeDisk = SystemDiskInfo.AvailableFreeBytes(exportRoot, out driveRoot);
            double cpu = SystemCpuInfo.CurrentUsagePercent();
            ulong ram = SystemMemoryInfo.AvailablePhysicalMemoryBytes();
            lock (sync)
            {
                if (cpu >= 0.0)
                {
                    sampledCpuSum += cpu;
                    sampledCpuCount++;
                }
                if (ram > 0 && ram < minimumAvailableRamBytes) minimumAvailableRamBytes = ram;
                if (freeDisk > 0 && freeDisk < minimumFreeDiskBytes) minimumFreeDiskBytes = freeDisk;
                AppendCsv(resourceCsv,
                    Csv(DateTime.UtcNow.ToString("o")), totalWatch.Elapsed.TotalSeconds.ToString("0.000", CultureInfo.InvariantCulture),
                    currentPhase.ToString(), currentCompleted.ToString(), currentTotal.ToString(), currentWorkerLimit.ToString(),
                    activeProcesses.ToString(), cpu < 0.0 ? "" : cpu.ToString("0.00", CultureInfo.InvariantCulture),
                    ram.ToString(), freeDisk.ToString());
            }
        }

        public void RecordFailure(int phase, BatchExportEntry entry, string message)
        {
            lock (sync)
            {
                AppendCsv(failuresCsv, Csv(DateTime.UtcNow.ToString("o")), phase.ToString(),
                    entry == null ? "" : entry.Order.ToString(), entry == null ? "" : Csv(Path.GetFileName(entry.DemoPath)), Csv(message));
            }
        }

        public void SetPhaseWallTime(int phase, double seconds)
        {
            lock (sync)
            {
                if (phase == 1) phase1Seconds = seconds;
                else if (phase == 2) phase2Seconds = seconds;
            }
        }

        public void Complete(string status, int candidateCount, IList<BatchExportEntry> entries,
            ResourcePlan plan, DiskPreflightEstimate preflight, string errorMessage)
        {
            lock (sync)
            {
                finalStatus = status;
                finalCandidateCount = candidateCount;
                totalWatch.Stop();
                Dictionary<string, object> summary = BuildBaseSummary(entries, plan, preflight);
                summary["status"] = finalStatus;
                summary["started_utc"] = startedUtc.ToString("o");
                summary["finished_utc"] = DateTime.UtcNow.ToString("o");
                summary["total_wall_seconds"] = totalWatch.Elapsed.TotalSeconds;
                summary["phase1_parse_wall_seconds"] = phase1Seconds;
                summary["phase2_analysis_wall_seconds"] = phase2Seconds;
                summary["candidate_count"] = finalCandidateCount;
                summary["successful_parse_jobs"] = successfulParseJobs;
                summary["successful_analysis_jobs"] = successfulAnalysisJobs;
                summary["total_parse_output_bytes"] = totalParseOutputBytes;
                summary["total_analysis_input_bytes"] = totalAnalysisInputBytes;
                summary["total_analysis_added_bytes"] = totalAnalysisAddedBytes;
                summary["total_parse_process_cpu_seconds"] = totalParseProcessCpuSeconds;
                summary["total_analysis_process_cpu_seconds"] = totalAnalysisProcessCpuSeconds;
                summary["max_parse_peak_working_set_bytes"] = maxParsePeakWorkingSetBytes;
                summary["max_analysis_peak_working_set_bytes"] = maxAnalysisPeakWorkingSetBytes;
                summary["average_sampled_system_cpu_percent"] = sampledCpuCount > 0 ? sampledCpuSum / sampledCpuCount : -1.0;
                summary["minimum_available_ram_bytes"] = minimumAvailableRamBytes == UInt64.MaxValue ? 0 : minimumAvailableRamBytes;
                summary["minimum_free_disk_bytes"] = minimumFreeDiskBytes == UInt64.MaxValue ? 0 : minimumFreeDiskBytes;
                summary["parse_source_mib_per_sec_wall"] = phase1Seconds > 0.0 ?
                    (Convert.ToDouble(summary["total_input_bytes"]) / BenchmarkFormatting.MiB) / phase1Seconds : 0.0;
                summary["parse_output_mib_per_sec_wall"] = phase1Seconds > 0.0 ?
                    (totalParseOutputBytes / BenchmarkFormatting.MiB) / phase1Seconds : 0.0;
                summary["analysis_input_mib_per_sec_wall"] = phase2Seconds > 0.0 ?
                    (totalAnalysisInputBytes / BenchmarkFormatting.MiB) / phase2Seconds : 0.0;
                summary["actual_export_bytes"] = BenchmarkFormatting.DirectorySize(exportRoot);
                summary["final_available_ram_bytes"] = SystemMemoryInfo.AvailablePhysicalMemoryBytes();
                string driveRoot;
                summary["final_free_disk_bytes"] = SystemDiskInfo.AvailableFreeBytes(exportRoot, out driveRoot);
                summary["error"] = errorMessage ?? "";
                summary["benchmark_directory"] = benchmarkDirectory;
                summary["persistent_history"] = history == null ? "" : history.HistoryPath;
                WriteJson(summaryJson, summary);
                if (history != null)
                {
                    Dictionary<string, object> batchRecord = new Dictionary<string, object>();
                    batchRecord["kind"] = "batch";
                    batchRecord["timestamp_utc"] = DateTime.UtcNow.ToString("o");
                    batchRecord["status"] = finalStatus;
                    batchRecord["demo_count"] = entries == null ? 0 : entries.Count;
                    batchRecord["total_input_bytes"] = summary["total_input_bytes"];
                    batchRecord["parse_workers"] = plan == null ? 0 : plan.ParseWorkers;
                    batchRecord["analysis_workers"] = plan == null ? 0 : plan.AnalysisWorkers;
                    batchRecord["phase1_parse_wall_seconds"] = phase1Seconds;
                    batchRecord["phase2_analysis_wall_seconds"] = phase2Seconds;
                    batchRecord["parse_source_mib_per_sec_wall"] = summary["parse_source_mib_per_sec_wall"];
                    batchRecord["parse_output_mib_per_sec_wall"] = summary["parse_output_mib_per_sec_wall"];
                    batchRecord["analysis_input_mib_per_sec_wall"] = summary["analysis_input_mib_per_sec_wall"];
                    batchRecord["average_sampled_system_cpu_percent"] = summary["average_sampled_system_cpu_percent"];
                    batchRecord["logical_processors"] = Environment.ProcessorCount;
                    batchRecord["total_physical_ram_bytes"] = SystemMemoryInfo.TotalPhysicalMemoryBytes();
                    batchRecord["processor_identifier"] = Environment.GetEnvironmentVariable("PROCESSOR_IDENTIFIER") ?? "";
                    history.AppendJobRecord(batchRecord);
                    try
                    {
                        if (File.Exists(history.HistoryPath))
                            File.Copy(history.HistoryPath, Path.Combine(benchmarkDirectory, "benchmark_history_after_run.ndjson"), true);
                    }
                    catch { }
                }
            }
        }

        public async Task SampleResourcesAsync(Func<int> activeProcessCount, CancellationToken cancellationToken)
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    RecordResourceSample(activeProcessCount == null ? 0 : activeProcessCount());
                }
                catch
                {
                    // Resource sampling is diagnostic only. A failed sample must never stop the batch.
                }

                try
                {
                    await Task.Delay(1000, cancellationToken);
                }
                catch (OperationCanceledException)
                {
                    break;
                }
            }
        }

        public void Dispose()
        {
            if (totalWatch.IsRunning) totalWatch.Stop();
        }

        private Dictionary<string, object> BuildBaseSummary(IList<BatchExportEntry> entries, ResourcePlan plan, DiskPreflightEstimate preflight)
        {
            Dictionary<string, object> summary = new Dictionary<string, object>();
            ulong totalInput = 0;
            int count = entries == null ? 0 : entries.Count;
            if (entries != null)
                foreach (BatchExportEntry entry in entries) totalInput += BenchmarkFormatting.FileSize(entry.DemoPath);

            summary["format"] = "tf2-frag-helper-benchmark";
            summary["format_version"] = 1;
            summary["demo_count"] = count;
            summary["total_input_bytes"] = totalInput;
            summary["os"] = Environment.OSVersion.ToString();
            summary["is_64bit_os"] = Environment.Is64BitOperatingSystem;
            summary["is_64bit_process"] = Environment.Is64BitProcess;
            summary["logical_processors"] = Environment.ProcessorCount;
            summary["processor_identifier"] = Environment.GetEnvironmentVariable("PROCESSOR_IDENTIFIER") ?? "";
            summary["initial_available_ram_bytes"] = plan == null ? SystemMemoryInfo.AvailablePhysicalMemoryBytes() : plan.AvailableMemoryBytes;
            summary["total_physical_ram_bytes"] = SystemMemoryInfo.TotalPhysicalMemoryBytes();
            summary["parse_workers"] = plan == null ? 0 : plan.ParseWorkers;
            summary["analysis_workers"] = plan == null ? 0 : plan.AnalysisWorkers;
            summary["estimated_parse_worker_ram_bytes"] = plan == null ? 0 : plan.EstimatedParseWorkerBytes;
            summary["estimated_analysis_worker_ram_bytes"] = plan == null ? 0 : plan.EstimatedAnalysisWorkerBytes;
            if (preflight != null)
            {
                summary["output_drive"] = preflight.DriveRoot;
                summary["initial_free_disk_bytes"] = preflight.AvailableFreeBytes;
                summary["estimated_parse_output_bytes"] = preflight.EstimatedParseOutputBytes;
                summary["estimated_analysis_added_bytes"] = preflight.EstimatedAnalysisAddedBytes;
                summary["estimated_output_bytes"] = preflight.EstimatedOutputBytes;
                summary["disk_safety_headroom_bytes"] = preflight.SafetyHeadroomBytes;
                summary["safe_required_free_bytes"] = preflight.SafeRequiredFreeBytes;
                summary["parse_expansion_ratio_used"] = preflight.ParseExpansionRatio;
                summary["analysis_expansion_ratio_used"] = preflight.AnalysisExpansionRatio;
                summary["historical_parse_samples"] = preflight.HistoricalParseSamples;
                summary["historical_analysis_samples"] = preflight.HistoricalAnalysisSamples;
            }
            return summary;
        }

        private Dictionary<string, object> CommonHistoryRecord(string kind, BatchExportEntry entry,
            WorkerRunResult result, int workerLimit)
        {
            Dictionary<string, object> record = new Dictionary<string, object>();
            record["kind"] = kind;
            record["timestamp_utc"] = DateTime.UtcNow.ToString("o");
            record["demo_name"] = Path.GetFileName(entry.DemoPath);
            record["wall_seconds"] = result.WallSeconds;
            record["cpu_seconds"] = result.CpuSeconds;
            record["peak_working_set_bytes"] = result.PeakWorkingSetBytes;
            record["worker_limit"] = workerLimit;
            record["logical_processors"] = Environment.ProcessorCount;
            record["total_physical_ram_bytes"] = SystemMemoryInfo.TotalPhysicalMemoryBytes();
            record["processor_identifier"] = Environment.GetEnvironmentVariable("PROCESSOR_IDENTIFIER") ?? "";
            return record;
        }

        private static ulong SafeAdd(ulong a, ulong b)
        {
            if (UInt64.MaxValue - a < b) return UInt64.MaxValue;
            return a + b;
        }

        private static void WriteHeader(string path, string header)
        {
            using (StreamWriter writer = new StreamWriter(path, false, new UTF8Encoding(false))) writer.WriteLine(header);
        }

        private static void AppendCsv(string path, params string[] fields)
        {
            using (StreamWriter writer = new StreamWriter(path, true, new UTF8Encoding(false)))
                writer.WriteLine(String.Join(",", fields));
        }

        private static string Csv(string value)
        {
            if (value == null) return "";
            return "\"" + value.Replace("\"", "\"\"").Replace("\r", " ").Replace("\n", " ") + "\"";
        }

        private static void WriteJson(string path, object value)
        {
            try
            {
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                serializer.MaxJsonLength = Int32.MaxValue;
                string json = serializer.Serialize(value);
                File.WriteAllText(path, PrettyJson(json), new UTF8Encoding(false));
            }
            catch { }
        }

        private static Dictionary<string, object> ReadJsonDictionary(string path)
        {
            try
            {
                if (!File.Exists(path)) return new Dictionary<string, object>();
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                serializer.MaxJsonLength = Int32.MaxValue;
                Dictionary<string, object> value = serializer.Deserialize<Dictionary<string, object>>(File.ReadAllText(path));
                return value ?? new Dictionary<string, object>();
            }
            catch
            {
                return new Dictionary<string, object>();
            }
        }

        private static IDictionary<string, object> DictionaryValue(IDictionary<string, object> source, string key)
        {
            if (source == null || !source.ContainsKey(key)) return new Dictionary<string, object>();
            IDictionary<string, object> value = source[key] as IDictionary<string, object>;
            return value ?? new Dictionary<string, object>();
        }

        private static string TextValue(IDictionary<string, object> source, string key)
        {
            if (source == null || !source.ContainsKey(key) || source[key] == null) return "";
            return Convert.ToString(source[key], CultureInfo.InvariantCulture) ?? "";
        }

        private static long LongValue(IDictionary<string, object> source, string key)
        {
            if (source == null || !source.ContainsKey(key) || source[key] == null) return 0;
            try { return Convert.ToInt64(source[key], CultureInfo.InvariantCulture); }
            catch { return 0; }
        }

        private static double DoubleValue(IDictionary<string, object> source, string key)
        {
            if (source == null || !source.ContainsKey(key) || source[key] == null) return 0.0;
            try { return Convert.ToDouble(source[key], CultureInfo.InvariantCulture); }
            catch { return 0.0; }
        }

        private static string PrettyJson(string json)
        {
            if (String.IsNullOrEmpty(json)) return json;
            StringBuilder output = new StringBuilder();
            bool quoted = false;
            bool escaped = false;
            int indent = 0;
            for (int i = 0; i < json.Length; i++)
            {
                char ch = json[i];
                if (quoted)
                {
                    output.Append(ch);
                    if (escaped) escaped = false;
                    else if (ch == '\\') escaped = true;
                    else if (ch == '"') quoted = false;
                    continue;
                }
                if (ch == '"') { quoted = true; output.Append(ch); }
                else if (ch == '{' || ch == '[')
                {
                    output.Append(ch).AppendLine();
                    indent++;
                    output.Append(new string(' ', indent * 2));
                }
                else if (ch == '}' || ch == ']')
                {
                    output.AppendLine();
                    indent = Math.Max(0, indent - 1);
                    output.Append(new string(' ', indent * 2)).Append(ch);
                }
                else if (ch == ',')
                {
                    output.Append(ch).AppendLine().Append(new string(' ', indent * 2));
                }
                else if (ch == ':') output.Append(": ");
                else output.Append(ch);
            }
            return output.ToString();
        }
    }

    internal sealed class AdaptiveDiskGate : IDisposable
    {
        private readonly object sync = new object();
        private readonly string outputPath;
        private readonly ulong minimumReserveBytes;
        private ulong reservedActiveBytes;
        private bool disposed;

        public AdaptiveDiskGate(string outputPath, ulong minimumReserveBytes)
        {
            this.outputPath = outputPath;
            this.minimumReserveBytes = minimumReserveBytes;
        }

        public async Task EnterAsync(ulong estimatedWriteBytes, CancellationToken cancellationToken)
        {
            while (true)
            {
                cancellationToken.ThrowIfCancellationRequested();
                bool allow = false;
                bool impossible = false;
                ulong free = 0;
                lock (sync)
                {
                    if (disposed) throw new ObjectDisposedException("AdaptiveDiskGate");
                    string driveRoot;
                    free = SystemDiskInfo.AvailableFreeBytes(outputPath, out driveRoot);
                    if (free == 0)
                    {
                        reservedActiveBytes = SafeAdd(reservedActiveBytes, estimatedWriteBytes);
                        allow = true;
                    }
                    else
                    {
                        ulong requiredForThisJob = SafeAdd(minimumReserveBytes, estimatedWriteBytes);
                        if (free < requiredForThisJob)
                            impossible = true;
                        else
                        {
                            ulong requiredWithReservations = SafeAdd(requiredForThisJob, reservedActiveBytes);
                            if (free >= requiredWithReservations)
                            {
                                reservedActiveBytes = SafeAdd(reservedActiveBytes, estimatedWriteBytes);
                                allow = true;
                            }
                        }
                    }
                }
                if (allow) return;
                if (impossible)
                    throw new IOException("Insufficient disk space to safely start the next job. Free " +
                        BenchmarkFormatting.Bytes(free) + "; this job plus safety reserve requires about " +
                        BenchmarkFormatting.Bytes(SafeAdd(minimumReserveBytes, estimatedWriteBytes)) + ". Completed exports were preserved.");
                await Task.Delay(500, cancellationToken);
            }
        }

        public void Exit(ulong estimatedWriteBytes)
        {
            lock (sync)
            {
                reservedActiveBytes = estimatedWriteBytes >= reservedActiveBytes ? 0 : reservedActiveBytes - estimatedWriteBytes;
            }
        }

        public void Dispose()
        {
            lock (sync) disposed = true;
        }

        private static ulong SafeAdd(ulong a, ulong b)
        {
            if (UInt64.MaxValue - a < b) return UInt64.MaxValue;
            return a + b;
        }
    }

    internal static class SystemDiskInfo
    {
        public static ulong AvailableFreeBytes(string path, out string driveRoot)
        {
            driveRoot = "";
            try
            {
                string full = Path.GetFullPath(path);
                string root = Path.GetPathRoot(full);
                driveRoot = root ?? "";
                if (String.IsNullOrWhiteSpace(root)) return 0;
                DriveInfo drive = new DriveInfo(root);
                if (!drive.IsReady) return 0;
                return (ulong)Math.Max(0L, drive.AvailableFreeSpace);
            }
            catch { return 0; }
        }
    }

    internal static class BenchmarkFormatting
    {
        public const double MiB = 1024.0 * 1024.0;
        public const double GiB = 1024.0 * 1024.0 * 1024.0;

        public static ulong FileSize(string path)
        {
            try { return File.Exists(path) ? (ulong)Math.Max(0L, new FileInfo(path).Length) : 0; }
            catch { return 0; }
        }

        public static ulong DirectorySize(string path)
        {
            if (String.IsNullOrWhiteSpace(path) || !Directory.Exists(path)) return 0;
            ulong total = 0;
            try
            {
                foreach (string file in Directory.EnumerateFiles(path, "*", SearchOption.AllDirectories))
                {
                    try
                    {
                        ulong size = (ulong)Math.Max(0L, new FileInfo(file).Length);
                        total = UInt64.MaxValue - total < size ? UInt64.MaxValue : total + size;
                    }
                    catch { }
                }
            }
            catch { }
            return total;
        }

        public static int CountNonEmptyLines(string path)
        {
            int count = 0;
            try
            {
                if (!File.Exists(path)) return 0;
                foreach (string line in File.ReadLines(path)) if (!String.IsNullOrWhiteSpace(line)) count++;
            }
            catch { }
            return count;
        }

        public static string Bytes(ulong bytes)
        {
            if (bytes >= (ulong)GiB) return (bytes / GiB).ToString("0.00") + " GB";
            if (bytes >= (ulong)MiB) return (bytes / MiB).ToString("0.0") + " MB";
            if (bytes >= 1024UL) return (bytes / 1024.0).ToString("0.0") + " KB";
            return bytes + " B";
        }

        public static string Duration(double seconds)
        {
            if (seconds < 0.0 || Double.IsNaN(seconds) || Double.IsInfinity(seconds)) return "unknown";
            TimeSpan span = TimeSpan.FromSeconds(seconds);
            if (span.TotalHours >= 1.0) return ((int)span.TotalHours) + "h " + span.Minutes + "m " + span.Seconds + "s";
            if (span.TotalMinutes >= 1.0) return ((int)span.TotalMinutes) + "m " + span.Seconds + "s";
            return Math.Max(0, (int)Math.Round(span.TotalSeconds)) + "s";
        }

        public static ulong ToUInt64Saturated(double value)
        {
            if (Double.IsNaN(value) || value <= 0.0) return 0;
            if (Double.IsInfinity(value) || value >= UInt64.MaxValue) return UInt64.MaxValue;
            return (ulong)Math.Ceiling(value);
        }
    }
}
