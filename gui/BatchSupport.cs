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
            if (String.Equals(value, "JPG image sequence", StringComparison.Ordinal)) return HlaeRecordingOutput.JpgSequence;
            if (String.Equals(value, "MP4 - standard", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Standard;
            if (String.Equals(value, "MP4 - compatible", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Compatible;
            if (String.Equals(value, "MP4 - lossless", StringComparison.Ordinal)) return HlaeRecordingOutput.Mp4Lossless;
            if (String.Equals(value, "AVI - raw", StringComparison.Ordinal)) return HlaeRecordingOutput.AviRaw;
            return HlaeRecordingOutput.TgaSequence;
        }

        public static string DisplayName(HlaeRecordingOutput output)
        {
            switch (output)
            {
                case HlaeRecordingOutput.JpgSequence: return "JPG image sequence";
                case HlaeRecordingOutput.Mp4Standard: return "MP4 - standard";
                case HlaeRecordingOutput.Mp4Compatible: return "MP4 - compatible";
                case HlaeRecordingOutput.Mp4Lossless: return "MP4 - lossless";
                case HlaeRecordingOutput.AviRaw: return "AVI - raw";
                default: return "TGA image sequence";
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

    internal sealed class HlaeRecordingSettings
    {
        public string FfmpegExecutable;
        public string HlaeExecutable;
        public string Tf2Executable;
        public string OutputDirectory;
    }

    internal sealed class HlaeRecordingSettingsForm : Form
    {
        private readonly TextBox hlaeBox = new TextBox();
        private readonly TextBox ffmpegBox = new TextBox();
        private readonly TextBox tf2Box = new TextBox();
        private readonly TextBox outputBox = new TextBox();

        public HlaeRecordingSettings Settings
        {
            get
            {
                return new HlaeRecordingSettings
                {
                    FfmpegExecutable = ffmpegBox.Text.Trim(),
                    HlaeExecutable = hlaeBox.Text.Trim(),
                    Tf2Executable = tf2Box.Text.Trim(),
                    OutputDirectory = outputBox.Text.Trim()
                };
            }
        }

        public HlaeRecordingSettingsForm(HlaeRecordingSettings initial)
        {
            Text = "HLAE batch recording setup (offline only)";
            StartPosition = FormStartPosition.CenterParent;
            MinimumSize = new System.Drawing.Size(760, 270);
            Size = new System.Drawing.Size(820, 290);
            Font = new System.Drawing.Font("Segoe UI", 9F);
            BackColor = System.Drawing.Color.FromArgb(30, 32, 36);
            ForeColor = System.Drawing.Color.Gainsboro;

            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.Padding = new Padding(14);
            layout.ColumnCount = 3;
            layout.RowCount = 6;
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 145));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 105));
            Controls.Add(layout);

            AddPathRow(layout, 0, "FFmpeg.exe", ffmpegBox, initial.FfmpegExecutable, BrowseFfmpeg);
            AddPathRow(layout, 1, "HLAE.exe", hlaeBox, initial.HlaeExecutable, BrowseHlae);
            AddPathRow(layout, 2, "TF2 executable", tf2Box, initial.Tf2Executable, BrowseTf2);
            AddPathRow(layout, 3, "Recording output", outputBox, initial.OutputDirectory, BrowseOutput);

            Label safety = new Label();
            safety.Text = "The app always launches HLAE with -insecure and +sv_lan 1, blocks connect/retry commands, and permits demo playback only.";
            safety.AutoSize = true;
            safety.ForeColor = System.Drawing.Color.LightGreen;
            safety.Margin = new Padding(3, 8, 3, 3);
            layout.SetColumnSpan(safety, 3);
            layout.Controls.Add(safety, 0, 4);

            FlowLayoutPanel buttons = new FlowLayoutPanel();
            buttons.Dock = DockStyle.Fill;
            buttons.FlowDirection = FlowDirection.RightToLeft;
            Button ok = new Button();
            ok.Text = "Prepare and launch";
            ok.Width = 140;
            ok.DialogResult = DialogResult.OK;
            Button cancel = new Button();
            cancel.Text = "Cancel";
            cancel.Width = 90;
            cancel.DialogResult = DialogResult.Cancel;
            buttons.Controls.Add(ok);
            buttons.Controls.Add(cancel);
            layout.SetColumnSpan(buttons, 3);
            layout.Controls.Add(buttons, 0, 5);
            AcceptButton = ok;
            CancelButton = cancel;
        }

        private static void AddPathRow(TableLayoutPanel layout, int row, string labelText, TextBox box, string value, EventHandler browse)
        {
            Label label = new Label();
            label.Text = labelText;
            label.AutoSize = true;
            label.Margin = new Padding(3, 9, 3, 3);
            layout.Controls.Add(label, 0, row);
            box.Text = value ?? "";
            box.Dock = DockStyle.Fill;
            layout.Controls.Add(box, 1, row);
            Button button = new Button();
            button.Text = "Browse";
            button.Width = 95;
            button.Click += browse;
            layout.Controls.Add(button, 2, row);
        }

        private void BrowseHlae(object sender, EventArgs e)
        {
            using (OpenFileDialog dialog = new OpenFileDialog())
            {
                dialog.Filter = "HLAE (HLAE.exe)|HLAE.exe|Executable (*.exe)|*.exe";
                if (File.Exists(hlaeBox.Text)) dialog.FileName = hlaeBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK)
                {
                    hlaeBox.Text = dialog.FileName;
                    if (!File.Exists(ffmpegBox.Text)) ffmpegBox.Text = HlaeBatchRecorder.FindFfmpegNearHlae(dialog.FileName) ?? "";
                }
            }
        }

        private void BrowseFfmpeg(object sender, EventArgs e)
        {
            using (OpenFileDialog dialog = new OpenFileDialog())
            {
                dialog.Filter = "FFmpeg (ffmpeg.exe)|ffmpeg.exe|Executable (*.exe)|*.exe";
                if (File.Exists(ffmpegBox.Text)) dialog.FileName = ffmpegBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK) ffmpegBox.Text = dialog.FileName;
            }
        }

        private void BrowseTf2(object sender, EventArgs e)
        {
            using (OpenFileDialog dialog = new OpenFileDialog())
            {
                dialog.Filter = "Team Fortress 2|tf_win64.exe;tf.exe|Executable (*.exe)|*.exe";
                if (File.Exists(tf2Box.Text)) dialog.FileName = tf2Box.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK) tf2Box.Text = dialog.FileName;
            }
        }

        private void BrowseOutput(object sender, EventArgs e)
        {
            using (FolderBrowserDialog dialog = new FolderBrowserDialog())
            {
                if (Directory.Exists(outputBox.Text)) dialog.SelectedPath = outputBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK) outputBox.Text = dialog.SelectedPath;
            }
        }
    }

    internal static class HlaeBatchRecorder
    {
        private const double TickRate = 66.6666667;

        private sealed class Clip
        {
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

        public static void Launch(Form owner, IList<IDictionary> selectedCandidates, string fallbackDemoPath, string suggestedTf2Executable, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            if (selectedCandidates == null || selectedCandidates.Count == 0)
                throw new InvalidOperationException("Select at least one candidate to record.");

            HlaeRecordingSettings initial = LoadSettings();
            if (String.IsNullOrEmpty(initial.Tf2Executable)) initial.Tf2Executable = suggestedTf2Executable;
            if (String.IsNullOrEmpty(initial.FfmpegExecutable)) initial.FfmpegExecutable = FindFfmpegNearHlae(initial.HlaeExecutable) ?? "";
            using (HlaeRecordingSettingsForm dialog = new HlaeRecordingSettingsForm(initial))
            {
                if (dialog.ShowDialog(owner) != DialogResult.OK) return;
                HlaeRecordingSettings settings = dialog.Settings;
                ValidateSettings(settings, output);
                SaveSettings(settings);
                PrepareAndLaunch(selectedCandidates, fallbackDemoPath, leadSeconds, outroSeconds, fps, output, jpgQuality, settings);
            }
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
            WriteOfflineConfig(gameDirectory);

            List<DemoQueue> demos = BuildQueue(selectedCandidates, fallbackDemoPath, leadSeconds, outroSeconds, outputDirectory, sessionId);
            if (demos.Count == 0) throw new InvalidOperationException("None of the selected candidates had a valid source demo and playback tick.");
            StageDemosAndWriteVdms(demos, stagedDirectory, gameDirectory, sessionId, fps, output, jpgQuality);
            WriteQueueManifest(demos, outputDirectory, leadSeconds, outroSeconds, fps, output, jpgQuality);

            string hlaeDirectory = Path.GetDirectoryName(settings.HlaeExecutable);
            bool x64 = Is64BitTf2(settings.Tf2Executable);
            string hook = x64 ? Path.Combine(hlaeDirectory, "x64", "AfxHookSource.dll") : Path.Combine(hlaeDirectory, "AfxHookSource.dll");
            string gameArguments = "-steam -insecure +sv_lan 1 -novid -window -console -no_texture_stream -afxGame tf " +
                (x64 ? "" : "-force32bit ") +
                "+tf_delete_temp_files 0 +exec tf2fragdemohelper_offline.cfg +playdemo " + demos[0].StagedRelativePath;
            string hlaeArguments = "-customLoader -autoStart -noGui " +
                "-programPath " + Quote(settings.Tf2Executable) + " " +
                "-cmdLine " + Quote(gameArguments) + " " +
                "-hookDllPath " + Quote(hook);

            DateTime launchTime = DateTime.Now;
            Process.Start(new ProcessStartInfo
            {
                FileName = settings.HlaeExecutable,
                Arguments = hlaeArguments,
                WorkingDirectory = hlaeDirectory,
                UseShellExecute = true
            });
            if (output == HlaeRecordingOutput.TgaSequence || output == HlaeRecordingOutput.JpgSequence)
                StartImageSequenceFinalizer(demos, gameDirectory, settings.Tf2Executable, launchTime, outputDirectory);
        }

        private static List<DemoQueue> BuildQueue(IList<IDictionary> selectedCandidates, string fallbackDemoPath, decimal leadSeconds, decimal outroSeconds, string outputDirectory, string sessionId)
        {
            Dictionary<string, DemoQueue> byDemo = new Dictionary<string, DemoQueue>(StringComparer.OrdinalIgnoreCase);
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

                DemoQueue demo;
                if (!byDemo.TryGetValue(demoPath, out demo))
                {
                    appearanceOrder++;
                    demo = new DemoQueue();
                    demo.DemoPath = demoPath;
                    demo.DemoName = Path.GetFileName(demoPath);
                    int recordedOrder = BatchCandidateSupport.CandidateDemoOrder(candidate);
                    demo.DemoOrder = recordedOrder > 0 ? recordedOrder : appearanceOrder;
                    byDemo.Add(demoPath, demo);
                }
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
            }

            List<DemoQueue> result = new List<DemoQueue>(byDemo.Values);
            result.Sort(delegate(DemoQueue left, DemoQueue right)
            {
                int order = left.DemoOrder.CompareTo(right.DemoOrder);
                return order != 0 ? order : String.Compare(left.DemoName, right.DemoName, StringComparison.OrdinalIgnoreCase);
            });
            int clipNumber = 0;
            foreach (DemoQueue demo in result)
            {
                demo.Clips.Sort(delegate(Clip left, Clip right) { return left.StartTick.CompareTo(right.StartTick); });
                foreach (Clip clip in demo.Clips)
                {
                    clipNumber++;
                    string name = clipNumber.ToString("D3") + "_" + BatchCandidateSupport.SafeName(Path.GetFileNameWithoutExtension(clip.DemoName)) + "_" + BatchCandidateSupport.SafeName(clip.CandidateId);
                    clip.OutputPath = Path.Combine(outputDirectory, name);
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
            int previousEnd = -1;
            foreach (Clip clip in demo.Clips)
            {
                if (clip.StartTick <= previousEnd)
                    clip.StartTick = previousEnd + 2;
                if (clip.EndTick <= clip.StartTick)
                    clip.EndTick = clip.StartTick + 1;

                int seekActionTick = previousEnd < 0 ? 1 : previousEnd + 1;
                AddSkipAction(lines, action++, seekActionTick, clip.StartTick);
                AddCommandAction(lines, action++, clip.StartTick + 1, "Record " + clip.CandidateId, "exec " + clip.StartConfigRelative);
                AddCommandAction(lines, action++, clip.EndTick, "Stop " + clip.CandidateId, "exec " + clip.StopConfigRelative);
                previousEnd = clip.EndTick;
            }
            string finishCommand = String.IsNullOrEmpty(nextDemo) ? "quit" : "playdemo " + nextDemo;
            AddCommandAction(lines, action++, previousEnd + recorderFlushTicks, "Continue batch", finishCommand);
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
                        BuildRecordingStopCommand(output, clip.CandidateId) + Environment.NewLine,
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
            return "echo TF2FRAG_RECORD_START " + ForwardSlashes(outputPath) + "; " +
                "host_framerate " + fps + "; mirv_streams record fps " + fps + "; " +
                "mirv_streams record screen enabled 1; " + RecordingProfileCommands(output, jpgQuality) +
                "mirv_streams record name \"" + ForwardSlashes(outputPath) + "\"; mirv_streams record start; hideconsole";
        }

        private static string BuildRecordingStopCommand(HlaeRecordingOutput output, string candidateId)
        {
            string stop = output == HlaeRecordingOutput.TgaSequence || output == HlaeRecordingOutput.JpgSequence
                ? "endmovie"
                : "mirv_streams record end";
            return "echo TF2FRAG_RECORD_END " + candidateId + "; " + stop + "; host_framerate 0";
        }

        private static void StartImageSequenceFinalizer(List<DemoQueue> demos, string gameDirectory, string tf2Executable, DateTime launchTime, string outputDirectory)
        {
            Thread worker = new Thread(delegate()
            {
                try
                {
                    WaitForTf2ToExit(tf2Executable, launchTime);
                    foreach (DemoQueue demo in demos)
                    {
                        foreach (Clip clip in demo.Clips)
                            TransferNativeMovieFiles(gameDirectory, clip);
                    }
                }
                catch (Exception error)
                {
                    try { File.WriteAllText(Path.Combine(outputDirectory, "recording_finalize_error.txt"), error.ToString(), new UTF8Encoding(false)); }
                    catch { }
                }
            });
            worker.IsBackground = true;
            worker.Name = "TF2 image sequence finalizer";
            worker.Start();
        }

        private static void WaitForTf2ToExit(string tf2Executable, DateTime launchTime)
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
            if (game == null) throw new InvalidOperationException("TF2 did not start, so image-sequence files could not be collected.");
            using (game) game.WaitForExit();
        }

        private static void TransferNativeMovieFiles(string gameDirectory, Clip clip)
        {
            string[] files = Directory.GetFiles(gameDirectory, clip.CaptureBaseName + "*");
            if (files.Length == 0)
                throw new FileNotFoundException("TF2 produced no startmovie files for " + clip.CandidateId + ". Check tf2fragdemohelper_recording.log for StartMovie errors.");
            foreach (string source in files)
            {
                string suffix = Path.GetFileName(source).Substring(clip.CaptureBaseName.Length);
                string destination = Path.Combine(clip.OutputPath, "frame" + suffix);
                File.Copy(source, destination, true);
                File.Delete(source);
            }
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

        private static void WriteQueueManifest(List<DemoQueue> demos, string outputDirectory, decimal leadSeconds, decimal outroSeconds, int fps, HlaeRecordingOutput output, int jpgQuality)
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            serializer.MaxJsonLength = Int32.MaxValue;
            Dictionary<string, object> manifest = new Dictionary<string, object>();
            manifest["format"] = "tf2-hlae-recording-queue";
            manifest["version"] = 1;
            manifest["offline_only"] = true;
            manifest["hlae_launch_flags"] = new string[] { "-insecure", "+sv_lan 1" };
            manifest["fps"] = fps;
            manifest["fps_semantics"] = "captured_frames_per_demo_second";
            manifest["output_format"] = HlaeRecordingOutputs.DisplayName(output);
            manifest["expected_output_files"] = HlaeRecordingOutputs.ExpectedFiles(output);
            if (output == HlaeRecordingOutput.JpgSequence) manifest["jpg_quality"] = jpgQuality;
            manifest["lead_in_seconds"] = leadSeconds;
            manifest["outro_seconds"] = outroSeconds;
            List<object> clips = new List<object>();
            foreach (DemoQueue demo in demos)
            {
                foreach (Clip clip in demo.Clips)
                {
                    Dictionary<string, object> record = new Dictionary<string, object>();
                    record["demo_order"] = demo.DemoOrder;
                    record["source_demo"] = demo.DemoPath;
                    record["candidate_id"] = clip.CandidateId;
                    record["start_tick"] = clip.StartTick;
                    record["end_tick"] = clip.EndTick;
                    record["attacker_user_id"] = clip.AttackerUserId;
                    record["output_path"] = clip.OutputPath;
                    record["native_capture_base"] = clip.CaptureBaseName;
                    clips.Add(record);
                }
            }
            manifest["clips"] = clips;
            File.WriteAllText(Path.Combine(outputDirectory, "recording_queue.json"), serializer.Serialize(manifest), new UTF8Encoding(false));
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
