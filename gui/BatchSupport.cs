using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace Tf2StvParserGui
{
    internal enum HlaeRecordingOutput
    {
        TgaSequence,
        JpgSequence,
        Mp4Standard,
        Mp4Compatible,
        Mp4Lossless,
        AviRaw
    }

    internal static class HlaeRecordingOutputs
    {
        public static HlaeRecordingOutput FromDisplayName(string value)
        {
            if (String.Equals(value, "JPG Image Sequence", StringComparison.Ordinal)) return HlaeRecordingOutput.JpgSequence;
            if (String.Equals(value, "MP4 - Standard", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Standard;
            if (String.Equals(value, "MP4 - Compatible", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Compatible;
            if (String.Equals(value, "MP4 - Lossless", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Lossless;
            if (String.Equals(value, "AVI - Raw", StringComparison.Ordinal)) return HlaeRecordingOutput.AviRaw;
            return HlaeRecordingOutput.TgaSequence;
        }

        public static string DisplayName(HlaeRecordingOutput output)
        {
            switch (output)
            {
                case HlaeRecordingOutput.JpgSequence: return "JPG Image Sequence";
                case HlaeRecordingOutput.Mp4Standard: return "MP4 - Standard";
                case HlaeRecordingOutput.Mp4Compatible: return "MP4 - Compatible";
                case HlaeRecordingOutput.Mp4Lossless: return "MP4 - Lossless";
                case HlaeRecordingOutput.AviRaw: return "AVI - Raw";
                default: return "TGA Image Sequence";
            }
        }

        public static bool RequiresFfmpeg(HlaeRecordingOutput output)
        {
            return output != HlaeRecordingOutput.TgaSequence && output != HlaeRecordingOutput.JpgSequence;
        }

        public static string ExpectedFiles(HlaeRecordingOutput output)
        {
            switch (output)
            {
                case HlaeRecordingOutput.JpgSequence: return "frame00000.jpg, frame00001.jpg, ...";
                case HlaeRecordingOutput.Mp4Standard:
                case HlaeRecordingOutput.Mp4Compatible:
                case HlaeRecordingOutput.Mp4Lossless: return "video.mp4";
                case HlaeRecordingOutput.AviRaw: return "video.avi";
                default: return "frame00000.tga, frame00001.tga, ...";
            }
        }
    }

    internal sealed class RecordingSizeEstimate
    {
        public int ClipCount;
        public double DurationSeconds;
        public long FrameCount;
        public double EstimatedBytes;
        public string Resolution;

        public string Summary(HlaeRecordingOutput output, int jpgQuality)
        {
            string quality = output == HlaeRecordingOutput.JpgSequence ? " at quality " + jpgQuality : "";
            return ClipCount + " selected clip" + (ClipCount == 1 ? "" : "s") + " | " +
                DurationSeconds.ToString("0.0") + " seconds | " + FrameCount.ToString("N0") +
                " frames | " + Resolution + " | " + HlaeRecordingOutputs.DisplayName(output) + quality +
                ": approximately " + HlaeRecordingSizeEstimator.FormatBytes(EstimatedBytes) +
                " (actual compressed-video size varies by scene).";
        }
    }

    internal static class HlaeRecordingSizeEstimator
    {
        private const double TickRate = 66.6666667;

        public static RecordingSizeEstimate Estimate(IList<IDictionary> candidates, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality, string resolution)
        {
            int width;
            int height;
            ParseResolution(resolution, out width, out height);
            RecordingSizeEstimate estimate = new RecordingSizeEstimate();
            estimate.Resolution = width + "x" + height;
            if (candidates == null) return estimate;
            double bytesPerFrame = EstimatedBytesPerFrame(width, height, output, jpgQuality);
            foreach (IDictionary candidate in candidates)
            {
                int firstTick = FirstEventTick(candidate);
                int lastTick = LastEventTick(candidate, firstTick);
                if (firstTick < 0 || lastTick < firstTick) continue;
                int startTick = Math.Max(0, firstTick - (int)Math.Round((double)leadSeconds * TickRate));
                int endTick = Math.Max(startTick + 1, lastTick + (int)Math.Round((double)outroSeconds * TickRate));
                double seconds = (endTick - startTick) / TickRate;
                long frames = Math.Max(1, (long)Math.Ceiling(seconds * fps));
                estimate.ClipCount++;
                estimate.DurationSeconds += seconds;
                estimate.FrameCount += frames;
            }
            estimate.EstimatedBytes = estimate.FrameCount * bytesPerFrame;
            return estimate;
        }

        public static string FormatBytes(double bytes)
        {
            string[] units = new string[] { "B", "KB", "MB", "GB", "TB" };
            int unit = 0;
            while (bytes >= 1024.0 && unit < units.Length - 1)
            {
                bytes /= 1024.0;
                unit++;
            }
            return bytes.ToString(bytes >= 100.0 || unit == 0 ? "0" : "0.0") + " " + units[unit];
        }

        private static double EstimatedBytesPerFrame(int width, int height, HlaeRecordingOutput output, int jpgQuality)
        {
            double pixels = width * height;
            switch (output)
            {
                case HlaeRecordingOutput.JpgSequence: return pixels * (0.025 + 0.0032 * Math.Max(1, Math.Min(100, jpgQuality)));
                case HlaeRecordingOutput.Mp4Standard: return pixels * 0.01125;
                case HlaeRecordingOutput.Mp4Compatible: return pixels * 0.00875;
                case HlaeRecordingOutput.Mp4Lossless: return pixels * 1.25;
                case HlaeRecordingOutput.AviRaw: return pixels * 3.0;
                default: return pixels * 3.0 + 18.0;
            }
        }

        private static void ParseResolution(string value, out int width, out int height)
        {
            width = 2560;
            height = 1440;
            if (String.IsNullOrEmpty(value)) return;
            string[] parts = value.ToLowerInvariant().Split('x');
            int parsedWidth;
            int parsedHeight;
            if (parts.Length == 2 && Int32.TryParse(parts[0].Trim(), out parsedWidth) && Int32.TryParse(parts[1].Trim(), out parsedHeight) && parsedWidth > 0 && parsedHeight > 0)
            {
                width = parsedWidth;
                height = parsedHeight;
            }
        }

        private static int FirstEventTick(IDictionary candidate)
        {
            IList ticks = candidate == null ? null : candidate["point_of_kill_ticks"] as IList;
            if (ticks != null && ticks.Count > 0) return IntValue(ticks[0], -1);
            return IntValue(candidate == null ? null : candidate["start_tick"], -1);
        }

        private static int LastEventTick(IDictionary candidate, int fallback)
        {
            IList ticks = candidate == null ? null : candidate["point_of_kill_ticks"] as IList;
            if (ticks != null && ticks.Count > 0) return IntValue(ticks[ticks.Count - 1], fallback);
            return IntValue(candidate == null ? null : candidate["end_tick"], fallback);
        }

        private static int IntValue(object value, int fallback)
        {
            try { return value == null ? fallback : Convert.ToInt32(value); }
            catch { return fallback; }
        }
    }

    internal sealed class BatchExportEntry
    {
        public readonly int Order;
        public readonly string DemoPath;
        public readonly string ExportDirectory;

        public BatchExportEntry(int order, string demoPath, string exportDirectory)
        {
            Order = order;
            DemoPath = demoPath;
            ExportDirectory = exportDirectory;
        }
    }

    internal static class BatchCandidateSupport
    {
        public static int WriteCombinedExport(string batchDirectory, IList<BatchExportEntry> exports)
        {
            JavaScriptSerializer serializer = NewSerializer();
            string candidatesPath = Path.Combine(batchDirectory, "frag_candidates.ndjson");
            int candidateCount = 0;
            using (StreamWriter writer = new StreamWriter(candidatesPath, false, new UTF8Encoding(false)))
            {
                foreach (BatchExportEntry entry in exports)
                {
                    string sourcePath = Path.Combine(entry.ExportDirectory, "frag_candidates.ndjson");
                    if (!File.Exists(sourcePath)) continue;
                    string mapName = ExportMapName(entry.ExportDirectory);
                    foreach (string line in File.ReadLines(sourcePath))
                    {
                        if (String.IsNullOrWhiteSpace(line)) continue;
                        IDictionary candidate = serializer.DeserializeObject(line) as IDictionary;
                        if (candidate == null) continue;
                        Dictionary<string, object> batchContext = new Dictionary<string, object>();
                        batchContext["demo_order"] = entry.Order;
                        batchContext["demo_name"] = Path.GetFileName(entry.DemoPath);
                        batchContext["source_demo"] = entry.DemoPath;
                        batchContext["source_export"] = entry.ExportDirectory;
                        batchContext["map_name"] = mapName;
                        candidate["batch_context"] = batchContext;
                        writer.WriteLine(serializer.Serialize(candidate));
                        candidateCount++;
                    }
                }
            }

            Dictionary<string, object> manifest = new Dictionary<string, object>();
            manifest["format"] = "tf2-frag-candidate-batch";
            manifest["version"] = 1;
            manifest["created_utc"] = DateTime.UtcNow.ToString("o");
            manifest["candidate_count"] = candidateCount;
            List<object> demoRecords = new List<object>();
            foreach (BatchExportEntry entry in exports)
            {
                Dictionary<string, object> record = new Dictionary<string, object>();
                record["demo_order"] = entry.Order;
                record["source_demo"] = entry.DemoPath;
                record["export_directory"] = entry.ExportDirectory;
                record["map_name"] = ExportMapName(entry.ExportDirectory);
                demoRecords.Add(record);
            }
            manifest["demos"] = demoRecords;
            File.WriteAllText(Path.Combine(batchDirectory, "manifest.json"), serializer.Serialize(manifest), new UTF8Encoding(false));
            return candidateCount;
        }

        public static string CandidateDemoPath(IDictionary candidate, string fallbackDemoPath)
        {
            IDictionary context = Value(candidate, "batch_context") as IDictionary;
            string source = TextValue(context, "source_demo");
            if (!String.IsNullOrEmpty(source)) return source;
            return fallbackDemoPath;
        }

        public static string CandidateDemoName(IDictionary candidate, string fallbackDemoPath)
        {
            IDictionary context = Value(candidate, "batch_context") as IDictionary;
            string name = TextValue(context, "demo_name");
            if (!String.IsNullOrEmpty(name)) return name;
            string path = CandidateDemoPath(candidate, fallbackDemoPath);
            return String.IsNullOrEmpty(path) ? "Unknown" : Path.GetFileName(path);
        }

        public static int CandidateDemoOrder(IDictionary candidate)
        {
            IDictionary context = Value(candidate, "batch_context") as IDictionary;
            return IntValue(context, "demo_order");
        }

        public static string CandidateMapName(IDictionary candidate, string fallbackExportDirectory)
        {
            string direct = TextValue(candidate, "map_name");
            if (String.IsNullOrWhiteSpace(direct)) direct = TextValue(candidate, "map");
            if (!String.IsNullOrWhiteSpace(direct)) return direct;
            IDictionary demoContext = Value(candidate, "demo_context") as IDictionary;
            direct = TextValue(demoContext, "map_name");
            if (String.IsNullOrWhiteSpace(direct)) direct = TextValue(demoContext, "map");
            if (!String.IsNullOrWhiteSpace(direct)) return direct;
            IDictionary context = Value(candidate, "batch_context") as IDictionary;
            string name = TextValue(context, "map_name");
            if (!String.IsNullOrWhiteSpace(name)) return name;

            string sourceExport = TextValue(context, "source_export");
            if (!String.IsNullOrWhiteSpace(sourceExport))
            {
                name = ExportMapName(sourceExport);
                if (!String.Equals(name, "Unknown", StringComparison.OrdinalIgnoreCase)) return name;
            }

            return ExportMapName(fallbackExportDirectory);
        }

        public static string ExportMapName(string exportDirectory)
        {
            if (String.IsNullOrWhiteSpace(exportDirectory) || !Directory.Exists(exportDirectory)) return "Unknown";
            JavaScriptSerializer serializer = NewSerializer();
            foreach (string fileName in new string[] { "header.json", "manifest.json" })
            {
                string path = Path.Combine(exportDirectory, fileName);
                if (!File.Exists(path)) continue;
                try
                {
                    IDictionary values = serializer.DeserializeObject(File.ReadAllText(path)) as IDictionary;
                    string name = TextValue(values, "map");
                    if (String.IsNullOrWhiteSpace(name)) name = TextValue(values, "map_name");
                    if (String.IsNullOrWhiteSpace(name)) name = TextValue(values, "mapname");
                    if (!String.IsNullOrWhiteSpace(name)) return name;
                }
                catch { }
            }
            return "Unknown";
        }

        public static string SafeName(string value)
        {
            StringBuilder result = new StringBuilder();
            foreach (char character in value ?? "")
            {
                if (Char.IsLetterOrDigit(character) || character == '_' || character == '-') result.Append(character);
                else result.Append('_');
            }
            return result.Length == 0 ? "item" : result.ToString();
        }

        private static JavaScriptSerializer NewSerializer()
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            serializer.MaxJsonLength = Int32.MaxValue;
            serializer.RecursionLimit = 256;
            return serializer;
        }

        private static object Value(IDictionary values, string key)
        {
            return values != null && values.Contains(key) ? values[key] : null;
        }

        private static string TextValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            return value == null ? "" : Convert.ToString(value);
        }

        private static int IntValue(IDictionary values, string key)
        {
            try { return Convert.ToInt32(Value(values, key)); }
            catch { return 0; }
        }
    }

    internal static class HlaeBatchRecorder
    {
        private const double TickRate = 66.6666667;
        private static readonly object ActiveRecordingLock = new object();
        private static Process activeHlaeProcess;
        private static string activeTf2Executable;
        private static DateTime activeLaunchTime;
        private static RecordingProfileSession activeProfileSession;
        private static Thread activeFinalizerThread;

        private sealed class Clip
        {
            public int Order;
            public IDictionary Candidate;
            public string CandidateId;
            public string DemoPath;
            public string DemoName;
            public int DemoOrder;
            public int StartTick;
            public int EndTick;
            public int AttackerUserId;
            public bool FocusAttacker;
            public string OutputPath;
            public string CaptureBaseName;
            public string StartConfigRelative;
            public string StopConfigRelative;
        }

        private sealed class DemoQueue
        {
            public string DemoPath;
            public string DemoName;
            public int DemoOrder;
            public string StagedRelativePath;
            public string StagedAbsolutePath;
            public readonly List<Clip> Clips = new List<Clip>();
        }

        private sealed class RecordingQueueTracker
        {
            private readonly object sync = new object();
            private readonly string queuePath;
            private readonly string manifestPath;
            private readonly Dictionary<string, object> root;
            private readonly Dictionary<string, Dictionary<string, object>> clips = new Dictionary<string, Dictionary<string, object>>(StringComparer.OrdinalIgnoreCase);

            public RecordingQueueTracker(string outputDirectory, Dictionary<string, object> manifest, IEnumerable<Dictionary<string, object>> clipRecords)
            {
                queuePath = Path.Combine(outputDirectory, "recording_queue.json");
                manifestPath = Path.Combine(outputDirectory, "recording_manifest.json");
                root = manifest;
                foreach (Dictionary<string, object> record in clipRecords)
                {
                    string capture = Convert.ToString(record["native_capture_base"]);
                    if (!String.IsNullOrEmpty(capture)) clips[capture] = record;
                }
                WriteLocked();
            }

            public void SetBatchStatus(string status, string reason)
            {
                lock (sync)
                {
                    root["batch_status"] = status;
                    root["updated_utc"] = DateTime.UtcNow.ToString("o");
                    if (!String.IsNullOrEmpty(reason)) root["status_reason"] = reason;
                    if (String.Equals(status, "Completed", StringComparison.OrdinalIgnoreCase) ||
                        String.Equals(status, "Failed", StringComparison.OrdinalIgnoreCase) ||
                        String.Equals(status, "Cancelled", StringComparison.OrdinalIgnoreCase))
                        root["completed_utc"] = DateTime.UtcNow.ToString("o");
                    WriteLocked();
                }
            }

            public void MarkRecording(string capture)
            {
                Transition(capture, "Recording", "recording_started_at", "");
            }

            public void MarkFinalizing(string capture)
            {
                Transition(capture, "Finalizing", "recording_stopped_at", "");
            }

            public void MarkCompleted(string capture, string actualOutputPath, long sizeBytes)
            {
                lock (sync)
                {
                    Dictionary<string, object> record;
                    if (!clips.TryGetValue(capture, out record)) return;
                    if (String.Equals(Convert.ToString(record["status"]), "Completed", StringComparison.OrdinalIgnoreCase)) return;
                    record["status"] = "Verified";
                    record["output_verified"] = true;
                    record["actual_output_path"] = actualOutputPath;
                    record["output_size_bytes"] = sizeBytes;
                    record["verified_at"] = DateTime.UtcNow.ToString("o");
                    record["status"] = "Completed";
                    record["completed_at"] = DateTime.UtcNow.ToString("o");
                    WriteLocked();
                }
            }

            public void MarkFailed(string capture, string reason)
            {
                lock (sync)
                {
                    Dictionary<string, object> record;
                    if (!clips.TryGetValue(capture, out record)) return;
                    if (String.Equals(Convert.ToString(record["status"]), "Completed", StringComparison.OrdinalIgnoreCase)) return;
                    record["status"] = "Failed";
                    record["output_verified"] = false;
                    record["failure_reason"] = reason;
                    record["completed_at"] = DateTime.UtcNow.ToString("o");
                    WriteLocked();
                }
            }

            public bool IsCompleted(string capture)
            {
                lock (sync)
                {
                    Dictionary<string, object> record;
                    return clips.TryGetValue(capture, out record) &&
                        String.Equals(Convert.ToString(record["status"]), "Completed", StringComparison.OrdinalIgnoreCase);
                }
            }

            public void FinishAfterProcessExit(bool normalExit)
            {
                bool allCompleted = true;
                lock (sync)
                {
                    foreach (Dictionary<string, object> record in clips.Values)
                    {
                        if (String.Equals(Convert.ToString(record["status"]), "Completed", StringComparison.OrdinalIgnoreCase)) continue;
                        allCompleted = false;
                        string current = Convert.ToString(record["status"]);
                        record["status"] = String.Equals(current, "Pending", StringComparison.OrdinalIgnoreCase) ? "Cancelled" : "Failed";
                        record["output_verified"] = false;
                        record["failure_reason"] = normalExit
                            ? "TF2 exited before this clip produced a verified output."
                            : "The recording process was interrupted before this clip produced a verified output.";
                        record["completed_at"] = DateTime.UtcNow.ToString("o");
                    }
                    root["batch_status"] = allCompleted ? "Completed" : "Failed";
                    root["updated_utc"] = DateTime.UtcNow.ToString("o");
                    root["completed_utc"] = DateTime.UtcNow.ToString("o");
                    if (!allCompleted) root["status_reason"] = "One or more clips did not produce a verified durable output.";
                    WriteLocked();
                }
            }

            private void Transition(string capture, string status, string timestampField, string reason)
            {
                lock (sync)
                {
                    Dictionary<string, object> record;
                    if (!clips.TryGetValue(capture, out record)) return;
                    string current = Convert.ToString(record["status"]);
                    if (String.Equals(current, "Completed", StringComparison.OrdinalIgnoreCase) ||
                        String.Equals(current, status, StringComparison.OrdinalIgnoreCase)) return;
                    record["status"] = status;
                    record[timestampField] = DateTime.UtcNow.ToString("o");
                    if (!String.IsNullOrEmpty(reason)) record["failure_reason"] = reason;
                    root["updated_utc"] = DateTime.UtcNow.ToString("o");
                    WriteLocked();
                }
            }

            private void WriteLocked()
            {
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                serializer.MaxJsonLength = Int32.MaxValue;
                string json = serializer.Serialize(root);
                AtomicWriteText(queuePath, json);
                AtomicWriteText(manifestPath, json);
            }
        }

        public static void Launch(Form owner, IList<IDictionary> selectedCandidates, string fallbackDemoPath, string suggestedTf2Executable, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            if (selectedCandidates == null || selectedCandidates.Count == 0)
                throw new InvalidOperationException("Select at least one candidate to record.");

            HlaeRecordingSettings initial = LoadSettings();
            if (String.IsNullOrEmpty(initial.Tf2Executable)) initial.Tf2Executable = suggestedTf2Executable;
            if (String.IsNullOrEmpty(initial.FfmpegExecutable)) initial.FfmpegExecutable = FindFfmpegNearHlae(initial.HlaeExecutable) ?? "";
            using (HlaeRecordingSettingsForm dialog = new HlaeRecordingSettingsForm(initial))
            {
                // These are preferences rather than a recording operation. Keep the current values
                // even when the user closes the setup window with Cancel or the title-bar button.
                dialog.FormClosing += delegate { SaveSettings(dialog.Settings); };
                if (dialog.ShowDialog(owner) != DialogResult.OK) return;
                HlaeRecordingSettings settings = dialog.Settings;
                ValidateSettings(settings, output);
                SaveSettings(settings);
                PrepareAndLaunch(selectedCandidates, fallbackDemoPath, leadSeconds, outroSeconds, fps, output, jpgQuality, settings);
            }
        }

        public static string SavedRecordingResolution()
        {
            return LoadSettings().Resolution;
        }

        private static void ValidateSettings(HlaeRecordingSettings settings, HlaeRecordingOutput output)
        {
            if (!File.Exists(settings.HlaeExecutable) || !String.Equals(Path.GetFileName(settings.HlaeExecutable), "HLAE.exe", StringComparison.OrdinalIgnoreCase))
                throw new FileNotFoundException("Select HLAE.exe from a current HLAE installation.", settings.HlaeExecutable);
            if (!File.Exists(settings.Tf2Executable))
                throw new FileNotFoundException("Select tf_win64.exe or tf.exe from Team Fortress 2.", settings.Tf2Executable);
            string tfName = Path.GetFileName(settings.Tf2Executable);
            if (!String.Equals(tfName, "tf_win64.exe", StringComparison.OrdinalIgnoreCase) && !String.Equals(tfName, "tf.exe", StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("The TF2 executable must be tf_win64.exe or tf.exe.");
            EnsureTf2IsNotRunning(settings.Tf2Executable);
            Directory.CreateDirectory(settings.OutputDirectory);
            if (!Directory.Exists(settings.OutputDirectory))
                throw new DirectoryNotFoundException("Choose a writable recording output folder.");

            string hlaeDirectory = Path.GetDirectoryName(settings.HlaeExecutable);
            string hook = Is64BitTf2(settings.Tf2Executable)
                ? Path.Combine(hlaeDirectory, "x64", "AfxHookSource.dll")
                : Path.Combine(hlaeDirectory, "AfxHookSource.dll");
            if (!File.Exists(hook))
                throw new FileNotFoundException(Is64BitTf2(settings.Tf2Executable)
                    ? "This HLAE installation does not contain x64\\AfxHookSource.dll. TF2 x64 requires HLAE 2.189.0 or newer."
                    : "This HLAE installation does not contain AfxHookSource.dll.", hook);
            if (HlaeRecordingOutputs.RequiresFfmpeg(output))
            {
                if (!File.Exists(settings.FfmpegExecutable) || !String.Equals(Path.GetFileName(settings.FfmpegExecutable), "ffmpeg.exe", StringComparison.OrdinalIgnoreCase))
                    throw new FileNotFoundException(HlaeRecordingOutputs.DisplayName(output) + " recording requires FFmpeg. Select ffmpeg.exe at the top of the setup window.", settings.FfmpegExecutable);
            }
        }

        internal static string FindFfmpegNearHlae(string hlaeExecutable)
        {
            if (String.IsNullOrEmpty(hlaeExecutable)) return null;
            string directory = Path.GetDirectoryName(hlaeExecutable);
            for (int level = 0; level < 3 && !String.IsNullOrEmpty(directory); level++)
            {
                string[] candidates = new string[]
                {
                    Path.Combine(directory, "ffmpeg", "bin", "ffmpeg.exe"),
                    Path.Combine(directory, "ffmpeg", "ffmpeg.exe"),
                    Path.Combine(directory, "HLAE FFMPEG", "ffmpeg", "bin", "ffmpeg.exe")
                };
                foreach (string candidate in candidates) if (File.Exists(candidate)) return candidate;
                directory = Path.GetDirectoryName(directory);
            }
            return null;
        }

        private static void EnsureTf2IsNotRunning(string tf2Executable)
        {
            string processName = Path.GetFileNameWithoutExtension(tf2Executable);
            foreach (Process process in Process.GetProcessesByName(processName))
            {
                try
                {
                    if (!process.HasExited)
                        throw new InvalidOperationException("Close TF2 before preparing a recording. This prevents the temporary offline recording profile from touching a running game.");
                }
                finally { process.Dispose(); }
            }
        }

        private static void PrepareAndLaunch(IList<IDictionary> selectedCandidates, string fallbackDemoPath, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality, HlaeRecordingSettings settings)
        {
            string tfRoot = Path.GetDirectoryName(settings.Tf2Executable);
            string gameDirectory = Path.Combine(tfRoot, "tf");
            if (!Directory.Exists(gameDirectory)) throw new DirectoryNotFoundException("Could not find TF2's tf folder next to the selected executable.");

            string sessionId = DateTime.Now.ToString("yyyyMMdd_HHmmss");
            string stagedDirectory = Path.Combine(gameDirectory, "demos", "tf2fragdemohelper_batch", sessionId);
            string outputDirectory = Path.Combine(settings.OutputDirectory, "tf2fragdemohelper_batch_" + sessionId);
            Directory.CreateDirectory(stagedDirectory);
            Directory.CreateDirectory(outputDirectory);
            RecordingProfileSession profile = null;
            try
            {
                profile = RecordingProfileManager.Apply(gameDirectory, sessionId, settings);
                WriteOfflineConfig(gameDirectory);

                List<DemoQueue> demos = BuildQueue(selectedCandidates, fallbackDemoPath, leadSeconds, outroSeconds, outputDirectory, sessionId);
                if (demos.Count == 0) throw new InvalidOperationException("None of the selected candidates had a valid source demo and playback tick.");
                StageDemosAndWriteVdms(demos, stagedDirectory, gameDirectory, sessionId, fps, output, jpgQuality);
                RecordingQueueTracker queueTracker = WriteQueueManifest(demos, outputDirectory, leadSeconds, outroSeconds, fps, output, jpgQuality, settings);

                string hlaeDirectory = Path.GetDirectoryName(settings.HlaeExecutable);
                bool x64 = Is64BitTf2(settings.Tf2Executable);
                string hook = x64 ? Path.Combine(hlaeDirectory, "x64", "AfxHookSource.dll") : Path.Combine(hlaeDirectory, "AfxHookSource.dll");
                int width;
                int height;
                ParseResolution(settings.Resolution, out width, out height);
                string dxArgument = DxLevelArgument(settings.DxLevel);
                string gameArguments = "-steam -insecure +sv_lan 1 -novid -window -noborder -console -no_texture_stream -afxGame tf " +
                    "-w " + width + " -h " + height + " " + dxArgument +
                    (x64 ? "" : "-force32bit ") +
                    "+tf_delete_temp_files 0 +exec tf2fragdemohelper_offline.cfg +exec tf2fragdemohelper_recording_profile.cfg +playdemo " + demos[0].StagedRelativePath;
                string hlaeArguments = "-customLoader -autoStart -noGui " +
                    "-programPath " + Quote(settings.Tf2Executable) + " " +
                    "-cmdLine " + Quote(gameArguments) + " " +
                    "-hookDllPath " + Quote(hook);

                DateTime launchTime = DateTime.Now;
                Process hlae = Process.Start(new ProcessStartInfo
                {
                    FileName = settings.HlaeExecutable,
                    Arguments = hlaeArguments,
                    WorkingDirectory = hlaeDirectory,
                    UseShellExecute = true
                });
                lock (ActiveRecordingLock)
                {
                    activeHlaeProcess = hlae;
                    activeTf2Executable = settings.Tf2Executable;
                    activeLaunchTime = launchTime;
                    activeProfileSession = profile;
                }
                queueTracker.SetBatchStatus("Running", "");
                StartRecordingFinalizer(demos, gameDirectory, settings.Tf2Executable, launchTime, outputDirectory, output, profile, queueTracker);
            }
            catch
            {
                if (profile != null) RecordingProfileManager.Restore(profile, true);
                throw;
            }
        }

        private static List<DemoQueue> BuildQueue(IList<IDictionary> selectedCandidates, string fallbackDemoPath, decimal leadSeconds, decimal outroSeconds, string outputDirectory, string sessionId)
        {
            List<DemoQueue> result = new List<DemoQueue>();
            int appearanceOrder = 0;
            foreach (IDictionary candidate in selectedCandidates)
            {
                string demoPath = BatchCandidateSupport.CandidateDemoPath(candidate, fallbackDemoPath);
                if (String.IsNullOrEmpty(demoPath) || !File.Exists(demoPath)) continue;
                IList ticks = Value(candidate, "point_of_kill_ticks") as IList;
                if (ticks == null || ticks.Count == 0) continue;
                int firstTick;
                int lastTick;
                try
                {
                    firstTick = Convert.ToInt32(ticks[0]);
                    lastTick = Convert.ToInt32(ticks[ticks.Count - 1]);
                }
                catch { continue; }

                appearanceOrder++;
                DemoQueue demo = new DemoQueue();
                demo.DemoPath = demoPath;
                demo.DemoName = Path.GetFileName(demoPath);
                int recordedOrder = BatchCandidateSupport.CandidateDemoOrder(candidate);
                demo.DemoOrder = recordedOrder > 0 ? recordedOrder : appearanceOrder;
                Clip clip = new Clip();
                clip.Candidate = candidate;
                clip.CandidateId = TextValue(candidate, "candidate_id");
                clip.DemoPath = demoPath;
                clip.DemoName = demo.DemoName;
                clip.DemoOrder = demo.DemoOrder;
                clip.StartTick = Math.Max(0, firstTick - (int)Math.Round((double)leadSeconds * TickRate));
                clip.EndTick = Math.Max(clip.StartTick + 1, lastTick + (int)Math.Round((double)outroSeconds * TickRate));
                clip.AttackerUserId = IntValue(candidate, "attacker_user_id");
                clip.FocusAttacker = IsStv(candidate) && clip.AttackerUserId > 0;
                demo.Clips.Add(clip);
                result.Add(demo);
            }

            result.Sort(delegate(DemoQueue left, DemoQueue right)
            {
                int order = left.DemoOrder.CompareTo(right.DemoOrder);
                if (order != 0) return order;
                int demoName = String.Compare(left.DemoName, right.DemoName, StringComparison.OrdinalIgnoreCase);
                if (demoName != 0) return demoName;
                return left.Clips[0].StartTick.CompareTo(right.Clips[0].StartTick);
            });
            int clipNumber = 0;
            foreach (DemoQueue demo in result)
            {
                foreach (Clip clip in demo.Clips)
                {
                    clipNumber++;
                    clip.Order = clipNumber;
                    string candidateName = String.IsNullOrWhiteSpace(clip.CandidateId) ? "candidate" : clip.CandidateId;
                    string name = clipNumber.ToString("D3") + "_" +
                        BatchCandidateSupport.SafeName(Path.GetFileNameWithoutExtension(clip.DemoName)) + "_" +
                        BatchCandidateSupport.SafeName(candidateName) + "_ticks_" + clip.StartTick + "-" + clip.EndTick;
                    clip.OutputPath = UniqueDirectoryPath(Path.Combine(outputDirectory, name));
                    clip.CaptureBaseName = "tf2frag_" + sessionId + "_" + clipNumber.ToString("D3");
                    Directory.CreateDirectory(clip.OutputPath);
                }
            }
            return result;
        }

        private static void StageDemosAndWriteVdms(List<DemoQueue> demos, string stagedDirectory, string gameDirectory, string sessionId, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            for (int index = 0; index < demos.Count; index++)
            {
                DemoQueue demo = demos[index];
                string stagedName = (index + 1).ToString("D3") + "_" + BatchCandidateSupport.SafeName(Path.GetFileNameWithoutExtension(demo.DemoName)) + ".dem";
                demo.StagedAbsolutePath = Path.Combine(stagedDirectory, stagedName);
                demo.StagedRelativePath = "demos/tf2fragdemohelper_batch/" + sessionId + "/" + stagedName;
                File.Copy(demo.DemoPath, demo.StagedAbsolutePath, true);
            }
            WriteRecordingConfigs(demos, gameDirectory, sessionId, fps, output, jpgQuality);
            for (int index = 0; index < demos.Count; index++)
            {
                string nextDemo = index + 1 < demos.Count ? demos[index + 1].StagedRelativePath : null;
                WriteRecordingVdm(demos[index], nextDemo);
            }
        }

        private static void WriteRecordingVdm(DemoQueue demo, string nextDemo)
        {
            int recorderFlushTicks = (int)Math.Round(TickRate * 2.0);
            List<string> lines = new List<string>();
            lines.Add("demoactions");
            lines.Add("{");
            int action = 1;
            AddCommandAction(lines, action++, 1, "Apply movie profile", "exec tf2fragdemohelper_recording_profile");
            int previousFinalizeTick = -1;
            foreach (Clip clip in demo.Clips)
            {
                if (clip.StartTick <= previousFinalizeTick)
                    clip.StartTick = previousFinalizeTick + 2;
                if (clip.EndTick <= clip.StartTick)
                    clip.EndTick = clip.StartTick + 1;

                int seekActionTick = previousFinalizeTick < 0 ? 2 : previousFinalizeTick + 2;
                AddSkipAction(lines, action++, seekActionTick, clip.StartTick);
                AddCommandAction(lines, action++, clip.StartTick + 1, "Record " + clip.CandidateId, "exec " + clip.StartConfigRelative);
                AddCommandAction(lines, action++, clip.EndTick, "Stop " + clip.CandidateId, "exec " + clip.StopConfigRelative);
                previousFinalizeTick = clip.EndTick + recorderFlushTicks;
                AddCommandAction(lines, action++, previousFinalizeTick, "Finalize " + clip.CandidateId,
                    "echo TF2FRAG_RECORD_FINALIZED " + clip.CaptureBaseName);
            }
            string finishCommand = String.IsNullOrEmpty(nextDemo) ? "quit" : "playdemo " + nextDemo;
            AddCommandAction(lines, action++, previousFinalizeTick + 2, "Continue batch", finishCommand);
            lines.Add("}");
            File.WriteAllLines(Path.ChangeExtension(demo.StagedAbsolutePath, ".vdm"), lines.ToArray(), new UTF8Encoding(false));
        }

        private static void WriteRecordingConfigs(List<DemoQueue> demos, string gameDirectory, string sessionId, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            string relativeDirectory = "tf2fragdemohelper_batch/" + sessionId;
            string configDirectory = Path.Combine(gameDirectory, "cfg", "tf2fragdemohelper_batch", sessionId);
            Directory.CreateDirectory(configDirectory);
            foreach (DemoQueue demo in demos)
            {
                foreach (Clip clip in demo.Clips)
                {
                    string startName = clip.CaptureBaseName + "_start";
                    string stopName = clip.CaptureBaseName + "_stop";
                    clip.StartConfigRelative = relativeDirectory + "/" + startName;
                    clip.StopConfigRelative = relativeDirectory + "/" + stopName;
                    string focus = clip.FocusAttacker
                        ? "spec_autodirector 0; spec_player #" + clip.AttackerUserId + "; spec_mode 4; "
                        : "";
                    File.WriteAllText(
                        Path.Combine(configDirectory, startName + ".cfg"),
                        focus + BuildRecordingStartCommand(clip.OutputPath, clip.CaptureBaseName, fps, output, jpgQuality) + Environment.NewLine,
                        new UTF8Encoding(false));
                    File.WriteAllText(
                        Path.Combine(configDirectory, stopName + ".cfg"),
                        BuildRecordingStopCommand(output, clip.CaptureBaseName) + Environment.NewLine,
                        new UTF8Encoding(false));
                }
            }
        }

        private static string BuildRecordingStartCommand(string outputPath, string captureBaseName, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            if (output == HlaeRecordingOutput.TgaSequence)
                return "echo TF2FRAG_RECORD_START " + captureBaseName + "; host_framerate " + fps + "; startmovie " + captureBaseName + " raw; hideconsole";
            if (output == HlaeRecordingOutput.JpgSequence)
                return "echo TF2FRAG_RECORD_START " + captureBaseName + "; jpeg_quality " + jpgQuality + "; host_framerate " + fps + "; startmovie " + captureBaseName + " jpeg; hideconsole";
            return "echo TF2FRAG_RECORD_START " + captureBaseName + "; " +
                "host_framerate " + fps + "; mirv_streams record fps " + fps + "; " +
                "mirv_streams record screen enabled 1; " + RecordingProfileCommands(output, jpgQuality) +
                "mirv_streams record name \"" + ForwardSlashes(outputPath) + "\"; mirv_streams record start; hideconsole";
        }

        private static string BuildRecordingStopCommand(HlaeRecordingOutput output, string captureBaseName)
        {
            string stop = output == HlaeRecordingOutput.TgaSequence || output == HlaeRecordingOutput.JpgSequence
                ? "endmovie"
                : "mirv_streams record end";
            return "echo TF2FRAG_RECORD_END " + captureBaseName + "; " + stop + "; host_framerate 0";
        }

        private static void StartRecordingFinalizer(List<DemoQueue> demos, string gameDirectory, string tf2Executable, DateTime launchTime, string outputDirectory, HlaeRecordingOutput output, RecordingProfileSession profile, RecordingQueueTracker queueTracker)
        {
            Thread worker = new Thread(delegate()
            {
                try
                {
                    MonitorRecordingSession(demos, gameDirectory, tf2Executable, launchTime, outputDirectory, output, queueTracker);
                }
                catch (Exception error)
                {
                    queueTracker.SetBatchStatus("Failed", error.Message);
                    try { File.WriteAllText(Path.Combine(outputDirectory, "recording_finalize_error.txt"), error.ToString(), new UTF8Encoding(false)); }
                    catch { }
                }
                finally
                {
                    RecordingProfileManager.Restore(profile, false);
                    lock (ActiveRecordingLock)
                    {
                        activeHlaeProcess = null;
                        activeTf2Executable = null;
                        activeProfileSession = null;
                        activeFinalizerThread = null;
                    }
                }
            });
            worker.IsBackground = true;
            worker.Name = "TF2 recording durability monitor";
            lock (ActiveRecordingLock) activeFinalizerThread = worker;
            worker.Start();
        }

        private static void MonitorRecordingSession(List<DemoQueue> demos, string gameDirectory, string tf2Executable, DateTime launchTime, string outputDirectory, HlaeRecordingOutput output, RecordingQueueTracker tracker)
        {
            Dictionary<string, Clip> clips = new Dictionary<string, Clip>(StringComparer.OrdinalIgnoreCase);
            foreach (DemoQueue demo in demos)
                foreach (Clip clip in demo.Clips)
                    clips[clip.CaptureBaseName] = clip;

            string logPath = Path.Combine(gameDirectory, "tf2fragdemohelper_recording.log");
            int processedLineCount = 0;
            Process game = FindTf2Process(tf2Executable, launchTime);
            using (game)
            {
                while (!game.HasExited)
                {
                    ProcessRecordingLog(logPath, ref processedLineCount, clips, gameDirectory, outputDirectory, output, tracker);
                    Thread.Sleep(250);
                }
                ProcessRecordingLog(logPath, ref processedLineCount, clips, gameDirectory, outputDirectory, output, tracker);
            }

            tracker.SetBatchStatus("Finalizing", "TF2 exited; verifying every selected clip.");
            foreach (Clip clip in clips.Values)
            {
                if (tracker.IsCompleted(clip.CaptureBaseName)) continue;
                TryFinalizeClip(clip, gameDirectory, outputDirectory, output, tracker, false);
            }
            tracker.FinishAfterProcessExit(true);
        }

        private static Process FindTf2Process(string tf2Executable, DateTime launchTime)
        {
            string processName = Path.GetFileNameWithoutExtension(tf2Executable);
            Process game = null;
            DateTime deadline = DateTime.Now.AddMinutes(2);
            while (DateTime.Now < deadline && game == null)
            {
                foreach (Process candidate in Process.GetProcessesByName(processName))
                {
                    try
                    {
                        if (candidate.StartTime >= launchTime.AddSeconds(-5)) { game = candidate; break; }
                    }
                    catch { candidate.Dispose(); }
                }
                if (game == null) Thread.Sleep(250);
            }
            if (game == null) throw new InvalidOperationException("TF2 did not start, so recording outputs could not be verified.");
            return game;
        }

        private static void ProcessRecordingLog(string logPath, ref int processedLineCount, IDictionary<string, Clip> clips, string gameDirectory, string outputDirectory, HlaeRecordingOutput output, RecordingQueueTracker tracker)
        {
            string[] lines;
            try
            {
                if (!File.Exists(logPath)) return;
                lines = File.ReadAllLines(logPath);
            }
            catch { return; }
            if (processedLineCount > lines.Length) processedLineCount = 0;
            for (int index = processedLineCount; index < lines.Length; index++)
            {
                string start = MarkerToken(lines[index], "TF2FRAG_RECORD_START ");
                string end = MarkerToken(lines[index], "TF2FRAG_RECORD_END ");
                string finalized = MarkerToken(lines[index], "TF2FRAG_RECORD_FINALIZED ");
                Clip clip;
                if (!String.IsNullOrEmpty(start) && clips.TryGetValue(start, out clip))
                {
                    tracker.MarkRecording(start);
                    AppendRecordingDiagnostic(outputDirectory, clip, "Recording", "recording start marker observed");
                }
                if (!String.IsNullOrEmpty(end) && clips.TryGetValue(end, out clip))
                {
                    tracker.MarkFinalizing(end);
                    AppendRecordingDiagnostic(outputDirectory, clip, "Finalizing", "recording stop marker observed");
                }
                if (!String.IsNullOrEmpty(finalized) && clips.TryGetValue(finalized, out clip))
                {
                    tracker.MarkFinalizing(finalized);
                    TryFinalizeClip(clip, gameDirectory, outputDirectory, output, tracker, true);
                }
            }
            processedLineCount = lines.Length;
        }

        private static string MarkerToken(string line, string marker)
        {
            int markerIndex = (line ?? "").IndexOf(marker, StringComparison.OrdinalIgnoreCase);
            if (markerIndex < 0) return "";
            string remainder = line.Substring(markerIndex + marker.Length).Trim();
            int whitespace = remainder.IndexOfAny(new char[] { ' ', '\t', '\r', '\n' });
            return whitespace < 0 ? remainder : remainder.Substring(0, whitespace);
        }

        private static void TryFinalizeClip(Clip clip, string gameDirectory, string outputDirectory, HlaeRecordingOutput output, RecordingQueueTracker tracker, bool waitForOutput)
        {
            if (tracker.IsCompleted(clip.CaptureBaseName)) return;
            try
            {
                tracker.MarkFinalizing(clip.CaptureBaseName);
                long outputSize;
                if (output == HlaeRecordingOutput.TgaSequence || output == HlaeRecordingOutput.JpgSequence)
                {
                    outputSize = TransferNativeMovieFiles(gameDirectory, clip, waitForOutput ? 15000 : 1000);
                }
                else
                {
                    outputSize = WaitForStableOutput(clip.OutputPath, waitForOutput ? 20000 : 3000);
                }
                if (outputSize <= 0) throw new IOException("The output exists but is empty.");
                tracker.MarkCompleted(clip.CaptureBaseName, clip.OutputPath, outputSize);
                AppendRecordingDiagnostic(outputDirectory, clip, "Completed", "verified durable output bytes=" + outputSize);
            }
            catch (Exception error)
            {
                tracker.MarkFailed(clip.CaptureBaseName, error.Message);
                AppendRecordingDiagnostic(outputDirectory, clip, "Failed", error.Message);
            }
        }

        private static long TransferNativeMovieFiles(string gameDirectory, Clip clip, int waitMilliseconds)
        {
            string[] files = new string[0];
            DateTime deadline = DateTime.Now.AddMilliseconds(Math.Max(0, waitMilliseconds));
            do
            {
                files = Directory.GetFiles(gameDirectory, clip.CaptureBaseName + "*");
                if (files.Length > 0) break;
                Thread.Sleep(250);
            }
            while (DateTime.Now < deadline);
            if (files.Length == 0)
                throw new FileNotFoundException("TF2 produced no startmovie files for " + clip.CandidateId + ". Check tf2fragdemohelper_recording.log for StartMovie errors.");
            long totalBytes = 0;
            foreach (string source in files)
            {
                string suffix = Path.GetFileName(source).Substring(clip.CaptureBaseName.Length);
                string destination = UniqueFilePath(Path.Combine(clip.OutputPath, "frame" + suffix));
                File.Copy(source, destination, false);
                totalBytes += new FileInfo(destination).Length;
                File.Delete(source);
            }
            return totalBytes;
        }

        private static long WaitForStableOutput(string outputPath, int waitMilliseconds)
        {
            DateTime deadline = DateTime.Now.AddMilliseconds(Math.Max(0, waitMilliseconds));
            long previousSize = -1;
            int stableSamples = 0;
            do
            {
                long size = DirectorySize(outputPath);
                if (size > 0 && size == previousSize) stableSamples++;
                else stableSamples = 0;
                if (stableSamples >= 3) return size;
                previousSize = size;
                Thread.Sleep(500);
            }
            while (DateTime.Now < deadline);
            long finalSize = DirectorySize(outputPath);
            if (finalSize <= 0) throw new FileNotFoundException("HLAE produced no non-empty encoded output in " + outputPath + ".");
            return finalSize;
        }

        private static long DirectorySize(string path)
        {
            if (String.IsNullOrEmpty(path) || !Directory.Exists(path)) return 0;
            long total = 0;
            foreach (string file in Directory.GetFiles(path, "*", SearchOption.AllDirectories))
            {
                try { total += Math.Max(0L, new FileInfo(file).Length); }
                catch { }
            }
            return total;
        }

        private static void AppendRecordingDiagnostic(string outputDirectory, Clip clip, string status, string detail)
        {
            try
            {
                string line = DateTime.UtcNow.ToString("o") + " | " + status + " | order=" + clip.Order +
                    " | candidate=" + clip.CandidateId + " | demo=" + clip.DemoPath +
                    " | ticks=" + clip.StartTick + "-" + clip.EndTick + " | expected=" + clip.OutputPath +
                    " | " + detail + Environment.NewLine;
                File.AppendAllText(Path.Combine(outputDirectory, "recording_finalize.log"), line, new UTF8Encoding(false));
            }
            catch { }
        }

        private static string RecordingProfileCommands(HlaeRecordingOutput output, int jpgQuality)
        {
            string setting = "afxClassic";
            if (output == HlaeRecordingOutput.Mp4Standard) setting = "afxFfmpeg";
            else if (output == HlaeRecordingOutput.Mp4Compatible) setting = "afxFfmpegYuv420p";
            else if (output == HlaeRecordingOutput.Mp4Lossless) setting = "afxFfmpegLosslessBest";
            else if (output == HlaeRecordingOutput.AviRaw) setting = "afxFfmpegRaw";
            return "mirv_streams record screen settings " + setting + "; ";
        }

        private static void AddSkipAction(List<string> lines, int action, int actionTick, int targetTick)
        {
            lines.Add("    \"" + action + "\"");
            lines.Add("    {");
            lines.Add("        factory \"SkipAhead\"");
            lines.Add("        name \"Batch seek\"");
            lines.Add("        starttick \"" + actionTick + "\"");
            lines.Add("        skiptotick \"" + targetTick + "\"");
            lines.Add("    }");
        }

        private static void AddCommandAction(List<string> lines, int action, int tick, string name, string commands)
        {
            lines.Add("    \"" + action + "\"");
            lines.Add("    {");
            lines.Add("        factory \"PlayCommands\"");
            lines.Add("        name \"" + EscapeVdm(name) + "\"");
            lines.Add("        starttick \"" + tick + "\"");
            lines.Add("        commands \"" + EscapeVdm(commands) + "\"");
            lines.Add("    }");
        }

        private static void WriteOfflineConfig(string gameDirectory)
        {
            string cfgDirectory = Path.Combine(gameDirectory, "cfg");
            Directory.CreateDirectory(cfgDirectory);
            List<string> lines = new List<string>(new string[]
            {
                "// Generated by TF2 Frag Demo Helper. Offline demo playback only.",
                "sv_lan 1",
                "cl_allowdownload 0",
                "cl_downloadfilter none",
                "alias connect \"echo BLOCKED: recording mode cannot connect to servers\"",
                "alias retry \"echo BLOCKED: recording mode cannot reconnect to servers\"",
                "alias tf_party_join_request_mode \"echo BLOCKED: matchmaking is disabled in recording mode\"",
                "engine_no_focus_sleep 0",
                "snd_mute_losefocus 0"
            });
            lines.Add("con_logfile tf2fragdemohelper_recording.log");
            lines.Add("echo TF2FRAG_RECORDER_INIT");
            lines.Add("echo TF2FRAG_RECORDER_READY");
            File.WriteAllLines(Path.Combine(cfgDirectory, "tf2fragdemohelper_offline.cfg"), lines.ToArray(), new UTF8Encoding(false));
        }

        private static RecordingQueueTracker WriteQueueManifest(List<DemoQueue> demos, string outputDirectory, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality, HlaeRecordingSettings settings)
        {
            Dictionary<string, object> manifest = new Dictionary<string, object>();
            manifest["format"] = "tf2-hlae-recording-queue";
            manifest["version"] = 2;
            manifest["batch_status"] = "Pending";
            manifest["offline_only"] = true;
            manifest["hlae_launch_flags"] = new string[] { "-insecure", "+sv_lan 1" };
            manifest["fps"] = fps;
            manifest["fps_semantics"] = "captured_frames_per_demo_second";
            manifest["output_format"] = HlaeRecordingOutputs.DisplayName(output);
            manifest["expected_output_files"] = HlaeRecordingOutputs.ExpectedFiles(output);
            if (output == HlaeRecordingOutput.JpgSequence) manifest["jpg_quality"] = jpgQuality;
            manifest["lead_in_seconds"] = leadSeconds;
            manifest["outro_seconds"] = outroSeconds;
            Dictionary<string, object> movieProfile = new Dictionary<string, object>();
            movieProfile["resolution"] = settings.Resolution;
            movieProfile["dx_level"] = settings.DxLevel;
            movieProfile["maximum_graphics"] = settings.MaximumGraphics;
            movieProfile["skybox"] = settings.Skybox;
            movieProfile["hud"] = settings.Hud;
            movieProfile["viewmodels"] = settings.Viewmodels;
            movieProfile["viewmodel_fov"] = settings.ViewmodelFov;
            movieProfile["motion_blur"] = settings.MotionBlur;
            movieProfile["isolated_custom_resources"] = settings.IsolateCustomResources;
            movieProfile["custom_resources"] = settings.CustomResources.ToArray();
            manifest["movie_profile"] = movieProfile;
            List<object> clipObjects = new List<object>();
            List<Dictionary<string, object>> clipRecords = new List<Dictionary<string, object>>();
            foreach (DemoQueue demo in demos)
            {
                foreach (Clip clip in demo.Clips)
                {
                    Dictionary<string, object> record = new Dictionary<string, object>();
                    record["order"] = clip.Order;
                    record["demo_order"] = demo.DemoOrder;
                    record["source_demo"] = demo.DemoPath;
                    record["candidate_id"] = clip.CandidateId;
                    record["start_tick"] = clip.StartTick;
                    record["end_tick"] = clip.EndTick;
                    record["attacker_user_id"] = clip.AttackerUserId;
                    record["output_path"] = clip.OutputPath;
                    record["expected_output_path"] = clip.OutputPath;
                    record["actual_output_path"] = null;
                    record["native_capture_base"] = clip.CaptureBaseName;
                    record["status"] = "Pending";
                    record["recording_started_at"] = null;
                    record["recording_stopped_at"] = null;
                    record["encoder_exit_code"] = null;
                    record["encoder_managed_by"] = HlaeRecordingOutputs.RequiresFfmpeg(output) ? "HLAE" : "TF2 startmovie";
                    record["output_verified"] = false;
                    record["output_size_bytes"] = 0;
                    record["completed_at"] = null;
                    clipRecords.Add(record);
                    clipObjects.Add(record);
                }
            }
            manifest["clips"] = clipObjects;
            return new RecordingQueueTracker(outputDirectory, manifest, clipRecords);
        }

        private static HlaeRecordingSettings LoadSettings()
        {
            HlaeRecordingSettings settings = new HlaeRecordingSettings();
            try
            {
                string path = SettingsPath();
                if (!File.Exists(path)) return settings;
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                IDictionary values = serializer.DeserializeObject(File.ReadAllText(path)) as IDictionary;
                settings.FfmpegExecutable = TextValue(values, "ffmpeg_executable");
                settings.HlaeExecutable = TextValue(values, "hlae_executable");
                settings.Tf2Executable = TextValue(values, "tf2_executable");
                settings.OutputDirectory = TextValue(values, "output_directory");
                settings.RecordingResourcesDirectory = TextValue(values, "recording_resources_directory");
                settings.Resolution = DefaultText(values, "resolution", settings.Resolution);
                settings.DxLevel = DefaultText(values, "dx_level", settings.DxLevel);
                settings.Skybox = DefaultText(values, "skybox", settings.Skybox);
                settings.Hud = DefaultText(values, "hud", settings.Hud);
                settings.Viewmodels = DefaultText(values, "viewmodels", settings.Viewmodels);
                settings.ViewmodelFov = DefaultInt(values, "viewmodel_fov", settings.ViewmodelFov);
                settings.MaximumGraphics = DefaultBool(values, "maximum_graphics", settings.MaximumGraphics);
                settings.MotionBlur = DefaultBool(values, "motion_blur", settings.MotionBlur);
                settings.DisableHitSounds = DefaultBool(values, "disable_hit_sounds", settings.DisableHitSounds);
                settings.DisableVoiceChat = DefaultBool(values, "disable_voice_chat", settings.DisableVoiceChat);
                settings.MinimalHud = DefaultBool(values, "minimal_hud", settings.MinimalHud);
                settings.DisableCombatText = DefaultBool(values, "disable_combat_text", settings.DisableCombatText);
                settings.DisableCrosshair = DefaultBool(values, "disable_crosshair", settings.DisableCrosshair);
                settings.DisableCrosshairSwitching = DefaultBool(values, "disable_crosshair_switching", settings.DisableCrosshairSwitching);
                settings.HudPlayerModel = DefaultBool(values, "hud_player_model", settings.HudPlayerModel);
                settings.IsolateCustomResources = DefaultBool(values, "isolate_custom_resources", settings.IsolateCustomResources);
                settings.DisableAnnouncerVoices = DefaultBool(values, "disable_announcer_voices", settings.DisableAnnouncerVoices);
                settings.DisableApplauseSounds = DefaultBool(values, "disable_applause_sounds", settings.DisableApplauseSounds);
                settings.DisableDominationSounds = DefaultBool(values, "disable_domination_sounds", settings.DisableDominationSounds);
                ReadStringList(values, "custom_resources", settings.CustomResources);
            }
            catch { }
            return settings;
        }

        private static void SaveSettings(HlaeRecordingSettings settings)
        {
            try
            {
                string path = SettingsPath();
                Directory.CreateDirectory(Path.GetDirectoryName(path));
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                Dictionary<string, object> values = new Dictionary<string, object>();
                values["ffmpeg_executable"] = settings.FfmpegExecutable;
                values["hlae_executable"] = settings.HlaeExecutable;
                values["tf2_executable"] = settings.Tf2Executable;
                values["output_directory"] = settings.OutputDirectory;
                values["recording_resources_directory"] = settings.RecordingResourcesDirectory;
                values["resolution"] = settings.Resolution;
                values["dx_level"] = settings.DxLevel;
                values["skybox"] = settings.Skybox;
                values["hud"] = settings.Hud;
                values["viewmodels"] = settings.Viewmodels;
                values["viewmodel_fov"] = settings.ViewmodelFov;
                values["maximum_graphics"] = settings.MaximumGraphics;
                values["motion_blur"] = settings.MotionBlur;
                values["disable_hit_sounds"] = settings.DisableHitSounds;
                values["disable_voice_chat"] = settings.DisableVoiceChat;
                values["minimal_hud"] = settings.MinimalHud;
                values["disable_combat_text"] = settings.DisableCombatText;
                values["disable_crosshair"] = settings.DisableCrosshair;
                values["disable_crosshair_switching"] = settings.DisableCrosshairSwitching;
                values["hud_player_model"] = settings.HudPlayerModel;
                values["isolate_custom_resources"] = settings.IsolateCustomResources;
                values["disable_announcer_voices"] = settings.DisableAnnouncerVoices;
                values["disable_applause_sounds"] = settings.DisableApplauseSounds;
                values["disable_domination_sounds"] = settings.DisableDominationSounds;
                values["custom_resources"] = settings.CustomResources.ToArray();
                File.WriteAllText(path, serializer.Serialize(values), new UTF8Encoding(false));
            }
            catch { }
        }

        private static string SettingsPath()
        {
            return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "TF2FragDemoHelper", "recording_settings.json");
        }

        private static bool Is64BitTf2(string path)
        {
            return String.Equals(Path.GetFileName(path), "tf_win64.exe", StringComparison.OrdinalIgnoreCase);
        }

        private static bool IsStv(IDictionary candidate)
        {
            IDictionary context = Value(candidate, "demo_context") as IDictionary;
            return String.Equals(TextValue(context, "capture_type"), "stv", StringComparison.OrdinalIgnoreCase);
        }

        private static object Value(IDictionary values, string key)
        {
            return values != null && values.Contains(key) ? values[key] : null;
        }

        private static string TextValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            return value == null ? "" : Convert.ToString(value);
        }

        private static int IntValue(IDictionary values, string key)
        {
            try { return Convert.ToInt32(Value(values, key)); }
            catch { return 0; }
        }

        public static void ShutdownActiveRecording()
        {
            Process hlae;
            string tf2Executable;
            DateTime launchTime;
            RecordingProfileSession profile;
            Thread finalizer;
            lock (ActiveRecordingLock)
            {
                hlae = activeHlaeProcess;
                tf2Executable = activeTf2Executable;
                launchTime = activeLaunchTime;
                profile = activeProfileSession;
                finalizer = activeFinalizerThread;
            }
            if (!String.IsNullOrEmpty(tf2Executable)) StopTf2Process(tf2Executable, launchTime);
            if (hlae != null)
            {
                try
                {
                    if (!hlae.HasExited)
                    {
                        hlae.CloseMainWindow();
                        if (!hlae.WaitForExit(3000)) hlae.Kill();
                    }
                }
                catch { }
                finally { hlae.Dispose(); }
            }
            if (finalizer != null && finalizer != Thread.CurrentThread)
            {
                try { finalizer.Join(20000); }
                catch { }
            }
            if (profile != null) RecordingProfileManager.Restore(profile, true);
            else RecordingProfileManager.RestoreActiveSession(true);
        }

        public static void RecoverInterruptedRecordings()
        {
            HlaeRecordingSettings settings = LoadSettings();
            if (settings == null || String.IsNullOrEmpty(settings.OutputDirectory) || !Directory.Exists(settings.OutputDirectory)) return;
            foreach (string directory in Directory.GetDirectories(settings.OutputDirectory, "tf2fragdemohelper_batch_*"))
            {
                string manifestPath = Path.Combine(directory, "recording_manifest.json");
                if (!File.Exists(manifestPath)) continue;
                try
                {
                    JavaScriptSerializer serializer = new JavaScriptSerializer();
                    serializer.MaxJsonLength = Int32.MaxValue;
                    IDictionary manifest = serializer.DeserializeObject(File.ReadAllText(manifestPath)) as IDictionary;
                    if (manifest == null) continue;
                    string batchStatus = TextValue(manifest, "batch_status");
                    if (String.Equals(batchStatus, "Completed", StringComparison.OrdinalIgnoreCase) ||
                        String.Equals(batchStatus, "Failed", StringComparison.OrdinalIgnoreCase) ||
                        String.Equals(batchStatus, "Cancelled", StringComparison.OrdinalIgnoreCase)) continue;
                    bool allCompleted = true;
                    IList clips = Value(manifest, "clips") as IList;
                    if (clips == null) continue;
                    foreach (object item in clips)
                    {
                        IDictionary record = item as IDictionary;
                        if (record == null) continue;
                        if (String.Equals(TextValue(record, "status"), "Completed", StringComparison.OrdinalIgnoreCase)) continue;
                        string expected = TextValue(record, "expected_output_path");
                        long size = Directory.Exists(expected) ? DirectorySize(expected) : (File.Exists(expected) ? new FileInfo(expected).Length : 0L);
                        if (size > 0)
                        {
                            record["status"] = "Completed";
                            record["output_verified"] = true;
                            record["actual_output_path"] = expected;
                            record["output_size_bytes"] = size;
                            record["verified_at"] = DateTime.UtcNow.ToString("o");
                            record["completed_at"] = DateTime.UtcNow.ToString("o");
                        }
                        else
                        {
                            allCompleted = false;
                            record["status"] = "Failed";
                            record["output_verified"] = false;
                            record["failure_reason"] = "The application previously closed before this recording produced a verifiable non-empty output.";
                            record["completed_at"] = DateTime.UtcNow.ToString("o");
                        }
                    }
                    manifest["batch_status"] = allCompleted ? "Completed" : "Failed";
                    manifest["updated_utc"] = DateTime.UtcNow.ToString("o");
                    manifest["completed_utc"] = DateTime.UtcNow.ToString("o");
                    if (!allCompleted) manifest["status_reason"] = "Recovered after an interrupted application session; incomplete clips were not replayed automatically.";
                    string json = serializer.Serialize(manifest);
                    AtomicWriteText(manifestPath, json);
                    string queuePath = Path.Combine(directory, "recording_queue.json");
                    if (File.Exists(queuePath)) AtomicWriteText(queuePath, json);
                }
                catch { }
            }
        }

        // Only paths under the helper's own names are removed here. Parsed
        // exports, source demos, and recorded video/frame folders are never
        // touched by this cleanup.
        public static void CleanupTemporaryFiles()
        {
            string restoreReason;
            if (!RecordingProfileManager.IsRestoreComplete(out restoreReason))
            {
                MessageBox.Show("Temporary recording cleanup was not started because restore verification has not completed.\r\n\r\n" + restoreReason,
                    "TF2 Frag Demo Helper restore warning", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            List<string> failures = new List<string>();
            string gameDirectory = null;
            lock (ActiveRecordingLock)
            {
                if (activeProfileSession != null) gameDirectory = activeProfileSession.TfDirectory;
            }
            HlaeRecordingSettings settings = LoadSettings();
            if (String.IsNullOrEmpty(gameDirectory) && settings != null && !String.IsNullOrEmpty(settings.Tf2Executable))
            {
                string tfRoot = Path.GetDirectoryName(settings.Tf2Executable);
                gameDirectory = String.IsNullOrEmpty(tfRoot) ? null : Path.Combine(tfRoot, "tf");
            }
            if (!String.IsNullOrEmpty(gameDirectory) && Directory.Exists(gameDirectory))
            {
                DeleteOwnedDirectory(Path.Combine(gameDirectory, "demos", "tf2fragdemohelper_batch"), failures);
                DeleteOwnedDirectory(Path.Combine(gameDirectory, "demos", "tf2fragdemohelper"), failures);
                DeleteOwnedDirectory(Path.Combine(gameDirectory, "cfg", "tf2fragdemohelper_batch"), failures);
                DeleteOwnedFile(Path.Combine(gameDirectory, "cfg", "tf2fragdemohelper_offline.cfg"), failures);
                DeleteOwnedFile(Path.Combine(gameDirectory, "cfg", "tf2fragdemohelper_recording_profile.cfg"), failures);
                DeleteOwnedFile(Path.Combine(gameDirectory, "tf2fragdemohelper_recording.log"), failures);
            }
            if (settings != null && !String.IsNullOrEmpty(settings.OutputDirectory) && Directory.Exists(settings.OutputDirectory))
            {
                foreach (string directory in Directory.GetDirectories(settings.OutputDirectory, "tf2fragdemohelper_batch_*"))
                {
                    DeleteOwnedFile(Path.Combine(directory, "recording_queue.json"), failures);
                    DeleteOwnedFile(Path.Combine(directory, "recording_queue.json.tmp"), failures);
                    DeleteOwnedFile(Path.Combine(directory, "recording_manifest.json.tmp"), failures);
                    DeleteOwnedFile(Path.Combine(directory, "recording_finalize.log"), failures);
                    DeleteOwnedFile(Path.Combine(directory, "recording_finalize_error.txt"), failures);
                }
            }
            if (failures.Count > 0)
                MessageBox.Show("Some helper-owned temporary files could not be removed:\r\n\r\n" + String.Join("\r\n", failures.ToArray()),
                    "TF2 Frag Demo Helper cleanup warning", MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }

        private static void DeleteOwnedDirectory(string path, IList<string> failures)
        {
            try
            {
                if (Directory.Exists(path)) Directory.Delete(path, true);
                if (Directory.Exists(path)) failures.Add(path);
            }
            catch { failures.Add(path); }
        }

        private static void DeleteOwnedFile(string path, IList<string> failures)
        {
            try
            {
                if (File.Exists(path)) File.Delete(path);
                if (File.Exists(path)) failures.Add(path);
            }
            catch { failures.Add(path); }
        }

        private static void StopTf2Process(string executable, DateTime launchTime)
        {
            string processName = Path.GetFileNameWithoutExtension(executable);
            foreach (Process process in Process.GetProcessesByName(processName))
            {
                try
                {
                    if (process.StartTime < launchTime.AddSeconds(-5)) continue;
                    process.CloseMainWindow();
                    if (!process.WaitForExit(5000)) process.Kill();
                }
                catch { }
                finally { process.Dispose(); }
            }
        }

        private static string UniqueDirectoryPath(string desiredPath)
        {
            string candidate = desiredPath;
            int suffix = 2;
            while (Directory.Exists(candidate) || File.Exists(candidate))
            {
                candidate = desiredPath + "_" + suffix;
                suffix++;
            }
            return candidate;
        }

        private static string UniqueFilePath(string desiredPath)
        {
            if (!File.Exists(desiredPath) && !Directory.Exists(desiredPath)) return desiredPath;
            string directory = Path.GetDirectoryName(desiredPath);
            string name = Path.GetFileNameWithoutExtension(desiredPath);
            string extension = Path.GetExtension(desiredPath);
            int suffix = 2;
            string candidate;
            do
            {
                candidate = Path.Combine(directory, name + "_" + suffix + extension);
                suffix++;
            }
            while (File.Exists(candidate) || Directory.Exists(candidate));
            return candidate;
        }

        private static void AtomicWriteText(string path, string contents)
        {
            string temporary = path + ".tmp";
            byte[] bytes = new UTF8Encoding(false).GetBytes(contents ?? "");
            using (FileStream stream = new FileStream(temporary, FileMode.Create, FileAccess.Write, FileShare.None))
            {
                stream.Write(bytes, 0, bytes.Length);
                stream.Flush(true);
            }
            if (!File.Exists(path))
            {
                File.Move(temporary, path);
                return;
            }
            try { File.Replace(temporary, path, null); }
            catch
            {
                File.Copy(temporary, path, true);
                File.Delete(temporary);
            }
        }

        private static void ParseResolution(string value, out int width, out int height)
        {
            width = 2560;
            height = 1440;
            string[] parts = (value ?? "").ToLowerInvariant().Split('x');
            if (parts.Length != 2) return;
            int parsedWidth;
            int parsedHeight;
            if (Int32.TryParse(parts[0], out parsedWidth) && Int32.TryParse(parts[1], out parsedHeight) && parsedWidth >= 640 && parsedHeight >= 360)
            {
                width = parsedWidth;
                height = parsedHeight;
            }
        }

        private static string DxLevelArgument(string value)
        {
            if (String.IsNullOrEmpty(value) || value.StartsWith("Default", StringComparison.OrdinalIgnoreCase)) return "";
            int space = value.IndexOf(' ');
            string level = space < 0 ? value : value.Substring(0, space);
            return "-dxlevel " + level + " ";
        }

        private static string DefaultText(IDictionary values, string key, string fallback)
        {
            string value = TextValue(values, key);
            return String.IsNullOrEmpty(value) ? fallback : value;
        }

        private static int DefaultInt(IDictionary values, string key, int fallback)
        {
            try { return values != null && values.Contains(key) ? Convert.ToInt32(values[key]) : fallback; }
            catch { return fallback; }
        }

        private static bool DefaultBool(IDictionary values, string key, bool fallback)
        {
            try { return values != null && values.Contains(key) ? Convert.ToBoolean(values[key]) : fallback; }
            catch { return fallback; }
        }

        private static void ReadStringList(IDictionary values, string key, IList<string> target)
        {
            if (values == null || !values.Contains(key)) return;
            IList list = values[key] as IList;
            if (list == null) return;
            target.Clear();
            foreach (object item in list) if (item != null) target.Add(Convert.ToString(item));
        }

        private static string EscapeVdm(string value)
        {
            return (value ?? "").Replace("\\", "\\\\").Replace("\"", "\\\"");
        }

        private static string ForwardSlashes(string value)
        {
            return (value ?? "").Replace('\\', '/');
        }

        private static string Quote(string value)
        {
            return "\"" + (value ?? "").Replace("\"", "\\\"") + "\"";
        }
    }
}
