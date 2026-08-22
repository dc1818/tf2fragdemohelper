using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace Tf2StvParserGui
{
    internal static class Program
    {
        [STAThread]
        private static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            RecordingProfileManager.RecoverInterruptedSession(null);
            HlaeBatchRecorder.RecoverInterruptedRecordings();
            MainForm main = new MainForm();
            main.FormClosing += delegate
            {
                HlaeBatchRecorder.ShutdownActiveRecording();
                HlaeBatchRecorder.CleanupTemporaryFiles();
            };
            Application.Run(main);
        }
    }

    internal sealed class MainForm : Form
    {
        private readonly string root;
        private readonly TextBox demoBox = new TextBox();
        private readonly TextBox outputBox = new TextBox();
        private readonly TextBox schemaBox = new TextBox();
        private readonly TextBox log = new TextBox();
        private readonly Button parseButton = GreenButton("Parse demo(s)", 150);
        private readonly Button cancelButton = GreenButton("Cancel", 90);
        private readonly Button openButton = GreenButton("Open export folder", 150);
        private readonly Button candidatesButton = GreenButton("View candidates", 130);
        private readonly Button loadExportButton = GreenButton("Load Previously Parsed Export", 210);
        private readonly ProgressBar progress = new ProgressBar();
        private readonly Label status = new Label();
        private TableLayoutPanel parserLayout;
        private CandidateViewerForm candidateViewer;
        private readonly object activeProcessesLock = new object();
        private readonly HashSet<Process> activeProcesses = new HashSet<Process>();
        private readonly object batchLogFileLock = new object();
        private string batchLogFilePath;
        private CancellationTokenSource batchCancellation;
        private bool busy;
        private string lastExport;
        private readonly List<string> selectedDemos = new List<string>();
        private string demoSelectionDisplay = "";
        private string autoDetectedSchemaPath = "";

        public MainForm()
        {
            root = AppDomain.CurrentDomain.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            Text = "TF2 STV Parser";
            StartPosition = FormStartPosition.CenterScreen;
            // The integrated candidate browser has a wide grid and a
            // dedicated Back button. Keep the shared window large enough for
            // those controls at normal Windows scaling.
            MinimumSize = new Size(1400, 760);
            Size = new Size(1480, 860);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;
            AllowDrop = true;
            DragEnter += OnDragEnter;
            DragDrop += OnDragDrop;
            BuildPage();
        }

        private static Button GreenButton(string text, int width)
        {
            Button button = new Button();
            button.Text = text;
            button.Width = width;
            button.Height = 36;
            button.FlatStyle = FlatStyle.Flat;
            button.BackColor = Color.FromArgb(44, 130, 82);
            button.ForeColor = Color.White;
            button.FlatAppearance.BorderColor = Color.FromArgb(64, 160, 103);
            return button;
        }

        private Label Label(string text)
        {
            Label label = new Label();
            label.Text = text;
            label.AutoSize = true;
            label.Margin = new Padding(3, 8, 3, 4);
            return label;
        }

        private void BuildPage()
        {
            parserLayout = new TableLayoutPanel();
            parserLayout.Dock = DockStyle.Fill;
            parserLayout.Padding = new Padding(20);
            parserLayout.ColumnCount = 3;
            parserLayout.RowCount = 8;
            parserLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 145));
            parserLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            parserLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 155));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 50));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            parserLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            Controls.Add(parserLayout);

            parserLayout.Controls.Add(Label("TF2 demo(s)"), 0, 0);
            demoBox.Dock = DockStyle.Fill;
            parserLayout.Controls.Add(demoBox, 1, 0);
            Button browseDemo = GreenButton("Browse demos", 145);
            browseDemo.Click += BrowseDemo;
            parserLayout.Controls.Add(browseDemo, 2, 0);

            parserLayout.Controls.Add(Label("Export location"), 0, 1);
            outputBox.Dock = DockStyle.Fill;
            outputBox.TextChanged += delegate { openButton.Enabled = Directory.Exists(outputBox.Text); };
            parserLayout.Controls.Add(outputBox, 1, 1);
            Button browseOutput = GreenButton("Select location", 145);
            browseOutput.Click += BrowseOutput;
            parserLayout.Controls.Add(browseOutput, 2, 1);

            parserLayout.Controls.Add(Label("Item schema (optional)"), 0, 2);
            schemaBox.Dock = DockStyle.Fill;
            parserLayout.Controls.Add(schemaBox, 1, 2);
            Button browseSchema = GreenButton("Select schema", 145);
            browseSchema.Click += BrowseSchema;
            parserLayout.Controls.Add(browseSchema, 2, 2);

            Label note = Label("Select one or more demos. Batch mode parses all demos first with automatic CPU/RAM-aware concurrency, then analyzes all parsed demos concurrently and creates one combined candidate list. Weapon slots use TF2's current items_game.txt when available.");
            note.MaximumSize = new Size(850, 0);
            note.ForeColor = Color.Silver;
            parserLayout.SetColumnSpan(note, 3);
            parserLayout.Controls.Add(note, 0, 3);

            Label logTitle = Label("Parser log");
            parserLayout.SetColumnSpan(logTitle, 3);
            parserLayout.Controls.Add(logTitle, 0, 4);
            log.Dock = DockStyle.Fill;
            log.Multiline = true;
            log.ReadOnly = true;
            log.ScrollBars = ScrollBars.Both;
            log.WordWrap = false;
            log.BackColor = Color.FromArgb(17, 18, 20);
            log.ForeColor = Color.FromArgb(218, 224, 230);
            log.Font = new Font("Consolas", 9F);
            parserLayout.SetColumnSpan(log, 3);
            parserLayout.Controls.Add(log, 0, 5);

            FlowLayoutPanel actions = new FlowLayoutPanel();
            actions.Dock = DockStyle.Fill;
            parseButton.Click += async delegate { await ParseDemo(); };
            cancelButton.Enabled = false;
            cancelButton.Click += Cancel;
            openButton.Enabled = false;
            openButton.Click += OpenExport;
            candidatesButton.Enabled = false;
            candidatesButton.Click += OpenCandidates;
            loadExportButton.Click += LoadParsedExport;
            actions.Controls.Add(parseButton);
            actions.Controls.Add(cancelButton);
            actions.Controls.Add(openButton);
            actions.Controls.Add(candidatesButton);
            actions.Controls.Add(loadExportButton);
            parserLayout.SetColumnSpan(actions, 3);
            parserLayout.Controls.Add(actions, 0, 6);

            status.Dock = DockStyle.Fill;
            parserLayout.SetColumnSpan(status, 3);
            parserLayout.Controls.Add(status, 0, 7);
            progress.Dock = DockStyle.Bottom;
            progress.Height = 14;
            progress.Style = ProgressBarStyle.Continuous;
            Controls.Add(progress);
        }

        private async Task ParseDemo()
        {
            if (busy) return;
            List<string> demos = SelectedDemoPaths();
            if (demos.Count == 0)
            {
                MessageBox.Show(this, "Choose one or more existing .dem files.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            if (String.IsNullOrWhiteSpace(outputBox.Text))
            {
                MessageBox.Show(this, "Choose an export location.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            string parser = Path.Combine(root, "parser", "target", "release", "export_all.exe");
            if (!File.Exists(parser))
            {
                MessageBox.Show(this, "The parser executable is missing. Run Build_Parser_GUI.bat once.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            string timestamp = DateTime.Now.ToString("yyyyMMdd_HHmmss");
            bool batch = demos.Count > 1;
            lastExport = batch
                ? Path.Combine(outputBox.Text.Trim(), "tf2_demo_batch_export_" + timestamp)
                : Path.Combine(outputBox.Text.Trim(), Path.GetFileNameWithoutExtension(demos[0]) + "_export_" + timestamp);
            Directory.CreateDirectory(lastExport);

            // Capture UI values before worker tasks begin. Worker threads should not read controls.
            string itemSchemaPath = String.IsNullOrWhiteSpace(schemaBox.Text) ? "" : schemaBox.Text.Trim();
            List<BatchExportEntry> batchExports = BuildBatchExportEntries(demos, batch, lastExport);
            BenchmarkHistory benchmarkHistory = BenchmarkHistory.Load();
            ResourcePlan resourcePlan = ResourcePlan.Create(demos, benchmarkHistory);
            DiskPreflightEstimate diskEstimate = benchmarkHistory.EstimateDisk(batchExports, lastExport);
            try
            {
                File.WriteAllText(Path.Combine(lastExport, "PRE_FLIGHT_ESTIMATE.txt"),
                    resourcePlan.Describe() + "\r\n" + diskEstimate.Describe(), new UTF8Encoding(false));
            }
            catch { }

            if (!diskEstimate.HasEstimatedOutputSpace)
            {
                MessageBox.Show(this,
                    "There is not enough free disk space for the estimated parsed output.\r\n\r\n" +
                    diskEstimate.Describe() +
                    "\r\nChoose an output drive with more free space before starting. The estimate is intentionally conservative and improves as benchmark history is collected.",
                    "Insufficient disk space", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return;
            }
            if (!diskEstimate.HasSafeSpace)
            {
                DialogResult proceed = MessageBox.Show(this,
                    "The batch probably fits, but the recommended safety headroom is not available. Running close to full can still cause OS error 112 if a demo expands more than expected.\r\n\r\n" +
                    diskEstimate.Describe() +
                    "\r\nContinue anyway?",
                    "Low disk headroom", MessageBoxButtons.YesNo, MessageBoxIcon.Warning);
                if (proceed != DialogResult.Yes) return;
            }
            else if (batch)
            {
                double predictedParseSeconds = resourcePlan.HistoricalParseSecondsPerGiB > 0.0
                    ? (diskEstimate.TotalInputBytes / BenchmarkFormatting.GiB) * resourcePlan.HistoricalParseSecondsPerGiB / Math.Max(1, resourcePlan.ParseWorkers)
                    : -1.0;
                double predictedAnalysisSeconds = resourcePlan.HistoricalAnalysisSecondsPerGiB > 0.0
                    ? (diskEstimate.EstimatedParseOutputBytes / BenchmarkFormatting.GiB) * resourcePlan.HistoricalAnalysisSecondsPerGiB / Math.Max(1, resourcePlan.AnalysisWorkers)
                    : -1.0;
                string historicalEta = predictedParseSeconds >= 0.0 && predictedAnalysisSeconds >= 0.0
                    ? "Historical batch ETA: about " + BenchmarkFormatting.Duration(predictedParseSeconds + predictedAnalysisSeconds) + "\r\n"
                    : "Historical batch ETA: calibrating; live ETA will appear after completed jobs and improve on later runs.\r\n";
                DialogResult startBatch = MessageBox.Show(this,
                    diskEstimate.Describe() + "\r\n" +
                    "Automatic parse workers: " + resourcePlan.ParseWorkers + "\r\n" +
                    "Automatic analysis workers: " + resourcePlan.AnalysisWorkers + "\r\n" +
                    historicalEta +
                    "\r\nStart this batch?",
                    "Batch preflight", MessageBoxButtons.YesNo, MessageBoxIcon.Information);
                if (startBatch != DialogResult.Yes) return;
            }

            busy = true;
            parseButton.Enabled = false;
            cancelButton.Enabled = true;
            openButton.Enabled = false;
            candidatesButton.Enabled = false;
            log.Clear();
            progress.Style = ProgressBarStyle.Continuous;
            progress.Minimum = 0;
            progress.Maximum = 100;
            progress.Value = 0;
            status.Text = "Phase 1 of 2: preparing to parse " + demos.Count + " demo" + (demos.Count == 1 ? "" : "s") + "...";
            status.ForeColor = Color.Gainsboro;

            CancellationTokenSource cancellation = new CancellationTokenSource();
            batchCancellation = cancellation;
            CancellationTokenSource samplerCancellation = new CancellationTokenSource();
            BenchmarkSession benchmark = new BenchmarkSession(lastExport, batchExports, resourcePlan, diskEstimate, benchmarkHistory);
            batchLogFilePath = Path.Combine(benchmark.DirectoryPath, "batch_run.log");
            Task samplerTask = Task.Run(() => benchmark.SampleResourcesAsync(GetActiveProcessCount, samplerCancellation.Token));
            int candidateCount = 0;

            Append("Inputs: " + demos.Count + " demo(s)\r\nExport: " + lastExport + "\r\n\r\n");
            Append(resourcePlan.Describe());
            Append("\r\n" + diskEstimate.Describe());
            Append("Benchmark/test data: " + benchmark.DirectoryPath + "\r\n");
            Append("Persistent calibration history: " + benchmarkHistory.HistoryPath + "\r\n");
            Append("\r\nPHASE 1 OF 2: PARSE ALL DEMOS\r\n");
            Append("Up to " + resourcePlan.ParseWorkers + " parser worker(s) will run concurrently.\r\n\r\n");

            try
            {
                await RunParsePhaseAsync(batchExports, parser, resourcePlan, diskEstimate, benchmark, cancellation.Token);

                cancellation.Token.ThrowIfCancellationRequested();
                Append("\r\nPHASE 1 COMPLETE: all " + demos.Count + " demo(s) parsed.\r\n");
                Append("\r\nPHASE 2 OF 2: ANALYZE ALL PARSED DEMOS\r\n");
                Append("Up to " + resourcePlan.AnalysisWorkers + " analyzer worker(s) will run concurrently.\r\n\r\n");

                await RunAnalysisPhaseAsync(batchExports, itemSchemaPath, resourcePlan, diskEstimate, benchmark, cancellation.Token);

                cancellation.Token.ThrowIfCancellationRequested();
                Append("\r\nPHASE 2 COMPLETE: all " + demos.Count + " demo(s) analyzed.\r\n");

                if (batch)
                {
                    status.Text = "Combining candidate results...";
                    candidateCount = BatchCandidateSupport.WriteCombinedExport(lastExport, batchExports);
                    Append("\r\nCombined " + candidateCount + " candidates from " + demos.Count + " demos into " + Path.Combine(lastExport, "frag_candidates.ndjson") + ".\r\n");
                }
                else
                {
                    candidateCount = BenchmarkFormatting.CountNonEmptyLines(Path.Combine(lastExport, "frag_candidates.ndjson"));
                }

                progress.Value = 100;
                status.Text = batch ? "Batch export and combined candidate analysis complete." : "Export and frag analysis complete.";
                status.ForeColor = Color.LightGreen;
                openButton.Enabled = true;
                candidatesButton.Enabled = true;
                benchmark.Complete("success", candidateCount, batchExports, resourcePlan, diskEstimate, "");
                Append("\r\nSUCCESS: Export and frag analysis complete. Use View candidates to inspect and batch-record ranked clips.\r\n");
                Append("Benchmark data was written to: " + benchmark.DirectoryPath + "\r\n");
            }
            catch (OperationCanceledException)
            {
                progress.Style = ProgressBarStyle.Continuous;
                status.Text = "Cancelled. Completed exports were left on disk.";
                status.ForeColor = Color.Goldenrod;
                openButton.Enabled = Directory.Exists(lastExport);
                benchmark.Complete("cancelled", candidateCount, batchExports, resourcePlan, diskEstimate, "Cancelled by user or batch cancellation.");
                Append("\r\nCANCELLED: active parser/analyzer workers were stopped. Completed export folders were preserved.\r\n");
                Append("Partial benchmark data was preserved in: " + benchmark.DirectoryPath + "\r\n");
            }
            catch (Exception ex)
            {
                progress.Style = ProgressBarStyle.Continuous;
                progress.Value = 0;
                status.Text = "Failed: " + ex.Message;
                status.ForeColor = Color.OrangeRed;
                openButton.Enabled = Directory.Exists(lastExport);
                benchmark.Complete("failed", candidateCount, batchExports, resourcePlan, diskEstimate, ex.ToString());
                Append("\r\nERROR: " + ex.Message + "\r\n");
                Append("Partial benchmark data was preserved in: " + benchmark.DirectoryPath + "\r\n");
            }
            finally
            {
                RequestStopActiveWorkers(false);
                samplerCancellation.Cancel();
                try { samplerTask.Wait(); } catch { }
                samplerCancellation.Dispose();
                benchmark.Dispose();
                batchLogFilePath = null;
                if (batchCancellation == cancellation) batchCancellation = null;
                cancellation.Dispose();
                busy = false;
                parseButton.Enabled = true;
                cancelButton.Enabled = false;
            }
        }

        private static List<BatchExportEntry> BuildBatchExportEntries(IList<string> demos, bool batch, string exportRoot)
        {
            List<BatchExportEntry> entries = new List<BatchExportEntry>();
            for (int index = 0; index < demos.Count; index++)
            {
                string demo = demos[index];
                string exportDirectory = batch
                    ? Path.Combine(exportRoot, (index + 1).ToString("D3") + "_" + SafeFileName(Path.GetFileNameWithoutExtension(demo)) + "_export")
                    : exportRoot;
                Directory.CreateDirectory(exportDirectory);
                entries.Add(new BatchExportEntry(index + 1, demo, exportDirectory));
            }
            return entries;
        }

        private async Task RunParsePhaseAsync(IList<BatchExportEntry> entries, string parser,
            ResourcePlan resourcePlan, DiskPreflightEstimate diskEstimate, BenchmarkSession benchmark,
            CancellationToken cancellationToken)
        {
            Stopwatch phaseWatch = Stopwatch.StartNew();
            int workerCount = resourcePlan.ParseWorkers;
            List<double> weights = new List<double>();
            foreach (BatchExportEntry entry in entries)
                weights.Add(Math.Max(1.0, (double)BenchmarkFormatting.FileSize(entry.DemoPath)));
            PhaseEtaTracker etaTracker = new PhaseEtaTracker(1, weights, resourcePlan.HistoricalParseSecondsPerGiB, workerCount);
            etaTracker.Start();
            benchmark.SetPhase(1, entries.Count, workerCount);
            EtaSnapshot initialEta = etaTracker.InitialSnapshot();
            benchmark.RecordEta(initialEta);
            Append("Phase 1 " + initialEta.ShortText() + "\r\n");

            ulong diskReserve = Math.Max(4UL * 1024UL * 1024UL * 1024UL,
                Math.Min(16UL * 1024UL * 1024UL * 1024UL, diskEstimate.SafetyHeadroomBytes / 4UL));
            try
            {
                using (SemaphoreSlim limiter = new SemaphoreSlim(workerCount, workerCount))
                using (AdaptiveResourceGate resourceGate = new AdaptiveResourceGate(resourcePlan.ReservedMemoryBytes))
                using (AdaptiveDiskGate diskGate = new AdaptiveDiskGate(lastExport, diskReserve))
                {
                    List<Task> tasks = new List<Task>();
                    foreach (BatchExportEntry entry in entries)
                    {
                        BatchExportEntry captured = entry;
                        tasks.Add(RunParseJobAsync(captured, parser, limiter, resourceGate, diskGate, resourcePlan, diskEstimate,
                            etaTracker, benchmark, workerCount, cancellationToken));
                    }
                    await Task.WhenAll(tasks);
                }
            }
            finally
            {
                phaseWatch.Stop();
                benchmark.SetPhaseWallTime(1, phaseWatch.Elapsed.TotalSeconds);
                Append("Phase 1 wall time: " + phaseWatch.Elapsed.TotalSeconds.ToString("0.0") + " seconds.\r\n");
            }
        }

        private async Task RunParseJobAsync(BatchExportEntry entry, string parser, SemaphoreSlim limiter,
            AdaptiveResourceGate resourceGate, AdaptiveDiskGate diskGate, ResourcePlan resourcePlan, DiskPreflightEstimate diskEstimate,
            PhaseEtaTracker etaTracker, BenchmarkSession benchmark, int workerLimit, CancellationToken cancellationToken)
        {
            await limiter.WaitAsync(cancellationToken);
            bool resourceSlot = false;
            bool diskSlot = false;
            ulong estimatedWriteBytes = benchmark.EstimateParseWriteBytes(entry, diskEstimate);
            try
            {
                ulong estimatedMemoryBytes = Math.Max(ResourcePlan.EstimateParseJobBytes(entry.DemoPath), resourcePlan.EstimatedParseWorkerBytes);
                await resourceGate.EnterAsync(estimatedMemoryBytes, cancellationToken);
                resourceSlot = true;
                await diskGate.EnterAsync(estimatedWriteBytes, cancellationToken);
                diskSlot = true;
                cancellationToken.ThrowIfCancellationRequested();

                string prefix = "[PARSE " + entry.Order.ToString("D3") + "] ";
                Append(prefix + "Starting " + Path.GetFileName(entry.DemoPath) +
                    " | estimated write " + BenchmarkFormatting.Bytes(estimatedWriteBytes) + "\r\n");
                Append(prefix + "Export: " + entry.ExportDirectory + "\r\n");

                WorkerRunResult result = await RunWorker(parser,
                    Quote(entry.DemoPath) + " " + Quote(entry.ExportDirectory), prefix, cancellationToken);
                ulong outputBytes = BenchmarkFormatting.DirectorySize(entry.ExportDirectory);
                benchmark.RecordParse(entry, result, outputBytes, workerLimit);

                EtaSnapshot eta = etaTracker.Complete(Math.Max(1.0, (double)BenchmarkFormatting.FileSize(entry.DemoPath)));
                benchmark.SetPhaseCompleted(eta.Completed);
                benchmark.RecordEta(eta);
                UpdatePhaseProgress(1, eta);

                Append(prefix + "Complete in " + result.WallSeconds.ToString("0.0") + " s: " +
                    Path.GetFileName(entry.DemoPath) + " | output " + BenchmarkFormatting.Bytes(outputBytes) +
                    " | " + eta.ShortText() + "\r\n");
            }
            catch (Exception ex)
            {
                benchmark.RecordFailure(1, entry, ex.Message);
                RequestStopActiveWorkers(true);
                throw;
            }
            finally
            {
                if (diskSlot) diskGate.Exit(estimatedWriteBytes);
                if (resourceSlot) resourceGate.Exit();
                limiter.Release();
            }
        }

        private async Task RunAnalysisPhaseAsync(IList<BatchExportEntry> entries, string itemSchemaPath,
            ResourcePlan resourcePlan, DiskPreflightEstimate diskEstimate, BenchmarkSession benchmark,
            CancellationToken cancellationToken)
        {
            Stopwatch phaseWatch = Stopwatch.StartNew();
            int workerCount = resourcePlan.AnalysisWorkers;
            List<double> weights = new List<double>();
            foreach (BatchExportEntry entry in entries)
            {
                ulong parseBytes = benchmark.ParseOutputBytes(entry.Order);
                if (parseBytes == 0) parseBytes = BenchmarkFormatting.DirectorySize(entry.ExportDirectory);
                weights.Add(Math.Max(1.0, (double)parseBytes));
            }
            PhaseEtaTracker etaTracker = new PhaseEtaTracker(2, weights, resourcePlan.HistoricalAnalysisSecondsPerGiB, workerCount);
            etaTracker.Start();
            benchmark.SetPhase(2, entries.Count, workerCount);
            EtaSnapshot initialEta = etaTracker.InitialSnapshot();
            benchmark.RecordEta(initialEta);
            Append("Phase 2 " + initialEta.ShortText() + "\r\n");

            ulong diskReserve = Math.Max(2UL * 1024UL * 1024UL * 1024UL,
                Math.Min(8UL * 1024UL * 1024UL * 1024UL, diskEstimate.SafetyHeadroomBytes / 8UL));
            try
            {
                using (SemaphoreSlim limiter = new SemaphoreSlim(workerCount, workerCount))
                using (AdaptiveResourceGate resourceGate = new AdaptiveResourceGate(resourcePlan.ReservedMemoryBytes))
                using (AdaptiveDiskGate diskGate = new AdaptiveDiskGate(lastExport, diskReserve))
                {
                    List<Task> tasks = new List<Task>();
                    foreach (BatchExportEntry entry in entries)
                    {
                        BatchExportEntry captured = entry;
                        tasks.Add(RunAnalysisJobAsync(captured, itemSchemaPath, limiter, resourceGate, diskGate,
                            resourcePlan, diskEstimate, etaTracker, benchmark, workerCount, cancellationToken));
                    }
                    await Task.WhenAll(tasks);
                }
            }
            finally
            {
                phaseWatch.Stop();
                benchmark.SetPhaseWallTime(2, phaseWatch.Elapsed.TotalSeconds);
                Append("Phase 2 wall time: " + phaseWatch.Elapsed.TotalSeconds.ToString("0.0") + " seconds.\r\n");
            }
        }

        private async Task RunAnalysisJobAsync(BatchExportEntry entry, string itemSchemaPath, SemaphoreSlim limiter,
            AdaptiveResourceGate resourceGate, AdaptiveDiskGate diskGate, ResourcePlan resourcePlan, DiskPreflightEstimate diskEstimate,
            PhaseEtaTracker etaTracker, BenchmarkSession benchmark, int workerLimit, CancellationToken cancellationToken)
        {
            await limiter.WaitAsync(cancellationToken);
            bool resourceSlot = false;
            bool diskSlot = false;
            ulong estimatedWriteBytes = benchmark.EstimateAnalysisWriteBytes(entry, diskEstimate);
            try
            {
                ulong estimatedMemoryBytes = Math.Max(ResourcePlan.EstimateAnalysisJobBytes(entry.DemoPath), resourcePlan.EstimatedAnalysisWorkerBytes);
                await resourceGate.EnterAsync(estimatedMemoryBytes, cancellationToken);
                resourceSlot = true;
                await diskGate.EnterAsync(estimatedWriteBytes, cancellationToken);
                diskSlot = true;
                cancellationToken.ThrowIfCancellationRequested();

                string prefix = "[ANALYZE " + entry.Order.ToString("D3") + "] ";
                // Divide the machine-wide CPU budget across the concurrently
                // active demo analyzers. The analyzer only uses these child
                // workers for independent candidate-group scoring; the large
                // StateTimeline remains in the parent process.
                int candidateWorkers = Math.Max(1,
                    Math.Min(8, Environment.ProcessorCount / Math.Max(1, workerLimit)));
                Append(prefix + "Starting candidate analysis for " + Path.GetFileName(entry.DemoPath) +
                    " | candidate scoring workers=" + candidateWorkers + "\r\n");
                WorkerRunResult result = await RunFragAnalysis(entry.ExportDirectory, itemSchemaPath, prefix, candidateWorkers, cancellationToken);
                ulong totalBytesAfter = BenchmarkFormatting.DirectorySize(entry.ExportDirectory);
                int candidateCount = BenchmarkFormatting.CountNonEmptyLines(Path.Combine(entry.ExportDirectory, "frag_candidates.ndjson"));
                benchmark.RecordAnalysis(entry, result, totalBytesAfter, candidateCount, workerLimit);

                ulong parseWeight = benchmark.ParseOutputBytes(entry.Order);
                if (parseWeight == 0) parseWeight = Math.Max(1UL, totalBytesAfter);
                EtaSnapshot eta = etaTracker.Complete(Math.Max(1.0, (double)parseWeight));
                benchmark.SetPhaseCompleted(eta.Completed);
                benchmark.RecordEta(eta);
                UpdatePhaseProgress(2, eta);

                Append(prefix + "Complete in " + result.WallSeconds.ToString("0.0") + " s: " +
                    Path.GetFileName(entry.DemoPath) + " | " + candidateCount + " candidate(s) | " + eta.ShortText() + "\r\n");
            }
            catch (Exception ex)
            {
                benchmark.RecordFailure(2, entry, ex.Message);
                RequestStopActiveWorkers(true);
                throw;
            }
            finally
            {
                if (diskSlot) diskGate.Exit(estimatedWriteBytes);
                if (resourceSlot) resourceGate.Exit();
                limiter.Release();
            }
        }

        private void UpdatePhaseProgress(int phase, EtaSnapshot eta)
        {
            if (InvokeRequired)
            {
                BeginInvoke(new Action<int, EtaSnapshot>(UpdatePhaseProgress), phase, eta);
                return;
            }

            double fraction = eta == null ? 0.0 : eta.Fraction;
            int percentWithinPhase = (int)Math.Round(Math.Max(0.0, Math.Min(1.0, fraction)) * 100.0);
            int overall = phase == 1 ? percentWithinPhase / 2 : 50 + (percentWithinPhase / 2);
            if (overall < 0) overall = 0;
            if (overall > 100) overall = 100;
            progress.Value = overall;
            string countText = eta == null ? "" : eta.Completed + " of " + eta.Total + " demos";
            string etaText = eta == null ? "ETA: calibrating..." : eta.ShortText();
            status.Text = "Phase " + phase + " of 2: " + (phase == 1 ? "parsed " : "analyzed ") +
                countText + ". " + etaText;
        }

        private Task<WorkerRunResult> RunWorker(string fileName, string arguments, string logPrefix, CancellationToken cancellationToken)
        {
            return Task.Run<WorkerRunResult>(delegate
            {
                cancellationToken.ThrowIfCancellationRequested();
                Process process = null;
                CancellationTokenRegistration cancellationRegistration = new CancellationTokenRegistration();
                Stopwatch wall = Stopwatch.StartNew();
                DateTime startedUtc = DateTime.UtcNow;
                try
                {
                    ProcessStartInfo info = new ProcessStartInfo();
                    info.FileName = fileName;
                    info.Arguments = arguments;
                    info.WorkingDirectory = root;
                    info.UseShellExecute = false;
                    info.CreateNoWindow = true;
                    info.RedirectStandardOutput = true;
                    info.RedirectStandardError = true;

                    process = new Process();
                    process.StartInfo = info;
                    Process capturedProcess = process;
                    process.OutputDataReceived += delegate(object sender, DataReceivedEventArgs e)
                    {
                        if (e.Data != null) Append(logPrefix + e.Data + "\r\n");
                    };
                    process.ErrorDataReceived += delegate(object sender, DataReceivedEventArgs e)
                    {
                        if (e.Data != null) Append(logPrefix + e.Data + "\r\n");
                    };

                    process.Start();
                    RegisterActiveProcess(process);
                    cancellationRegistration = cancellationToken.Register(delegate { TryKillProcess(capturedProcess); });
                    process.BeginOutputReadLine();
                    process.BeginErrorReadLine();
                    process.WaitForExit();
                    wall.Stop();
                    int code = process.ExitCode;

                    cancellationToken.ThrowIfCancellationRequested();
                    if (code != 0)
                        throw new InvalidOperationException(logPrefix + "Worker exited with code " + code + ". See the prefixed log lines above.");

                    WorkerRunResult result = new WorkerRunResult();
                    result.StartedUtc = startedUtc;
                    result.FinishedUtc = DateTime.UtcNow;
                    result.WallSeconds = wall.Elapsed.TotalSeconds;
                    result.ExitCode = code;
                    result.Executable = fileName;
                    try { result.CpuSeconds = process.TotalProcessorTime.TotalSeconds; } catch { result.CpuSeconds = 0.0; }
                    try { result.PeakWorkingSetBytes = process.PeakWorkingSet64; } catch { result.PeakWorkingSetBytes = 0; }
                    return result;
                }
                finally
                {
                    wall.Stop();
                    cancellationRegistration.Dispose();
                    if (process != null)
                    {
                        UnregisterActiveProcess(process);
                        process.Dispose();
                    }
                }
            }, cancellationToken);
        }

        private async Task<WorkerRunResult> RunFragAnalysis(string exportDirectory, string itemSchemaPath, string logPrefix, int candidateWorkers, CancellationToken cancellationToken)
        {
            string script = Path.Combine(root, "analyze_frags.py");
            if (!File.Exists(script)) throw new FileNotFoundException("Frag analyzer is missing.", script);
            // Normal batch analysis writes analysis_profile.json automatically.
            // Do not enable per-event --debug logging here: large POV/STV demos
            // can contain thousands of intentionally rejected deaths (warmup,
            // post-round, or non-POV attackers), and printing each rejection is
            // measurable console/process overhead. Run analyze_frags.py manually
            // with --debug only when investigating a specific demo.
            string analyzerArguments = Quote(script) + " --candidate-workers " + Math.Max(1, candidateWorkers).ToString() + " ";
            if (!String.IsNullOrWhiteSpace(itemSchemaPath))
                analyzerArguments += "--item-schema " + Quote(itemSchemaPath) + " ";
            analyzerArguments += Quote(exportDirectory);

            Exception pythonFailure = null;
            try
            {
                return await RunWorker("python.exe", analyzerArguments, logPrefix, cancellationToken);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex)
            {
                pythonFailure = ex;
                Append(logPrefix + "python.exe failed; trying py.exe -3. " + ex.Message + "\r\n");
            }

            if (pythonFailure != null)
                return await RunWorker("py.exe", "-3 " + analyzerArguments, logPrefix, cancellationToken);
            throw new InvalidOperationException(logPrefix + "Candidate analyzer did not run.");
        }

        private int GetActiveProcessCount()
        {
            lock (activeProcessesLock) return activeProcesses.Count;
        }

        private void RegisterActiveProcess(Process process)
        {
            lock (activeProcessesLock)
            {
                activeProcesses.Add(process);
            }
        }

        private void UnregisterActiveProcess(Process process)
        {
            lock (activeProcessesLock)
            {
                activeProcesses.Remove(process);
            }
        }

        private static void TryKillProcess(Process process)
        {
            if (process == null) return;
            try
            {
                if (process.HasExited) return;

                // Candidate analysis may temporarily use bounded child Python
                // processes for independent group scoring. Kill the complete
                // process tree on Windows so Cancel does not leave orphaned
                // scoring workers behind. Fall back to Process.Kill if
                // taskkill is unavailable for any reason.
                try
                {
                    ProcessStartInfo killInfo = new ProcessStartInfo();
                    killInfo.FileName = "taskkill.exe";
                    killInfo.Arguments = "/PID " + process.Id.ToString() + " /T /F";
                    killInfo.UseShellExecute = false;
                    killInfo.CreateNoWindow = true;
                    using (Process killer = Process.Start(killInfo))
                    {
                        if (killer != null) killer.WaitForExit(5000);
                    }
                }
                catch
                {
                    if (!process.HasExited) process.Kill();
                }
            }
            catch
            {
            }
        }

        private void RequestStopActiveWorkers(bool cancelBatch)
        {
            CancellationTokenSource cancellation = batchCancellation;
            if (cancelBatch && cancellation != null && !cancellation.IsCancellationRequested)
            {
                try { cancellation.Cancel(); }
                catch { }
            }

            List<Process> processes;
            lock (activeProcessesLock)
            {
                processes = new List<Process>(activeProcesses);
            }

            foreach (Process process in processes)
                TryKillProcess(process);
        }

        private void Cancel(object sender, EventArgs e)
        {
            if (!busy) return;
            cancelButton.Enabled = false;
            status.Text = "Cancelling active parser/analyzer workers and queued jobs...";
            RequestStopActiveWorkers(true);
        }

        private void BrowseDemo(object sender, EventArgs e)
        {
            using (OpenFileDialog dialog = new OpenFileDialog())
            {
                dialog.Filter = "TF2 demo (*.dem)|*.dem|All files (*.*)|*.*";
                dialog.Multiselect = true;
                dialog.Title = "Select one or more TF2 demos";
                if (dialog.ShowDialog(this) == DialogResult.OK) SetSelectedDemos(dialog.FileNames);
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

        private void BrowseSchema(object sender, EventArgs e)
        {
            using (OpenFileDialog dialog = new OpenFileDialog())
            {
                dialog.Filter = "TF2 item schema (items_game.txt)|items_game.txt|Text files (*.txt)|*.txt|All files (*.*)|*.*";
                if (File.Exists(schemaBox.Text)) dialog.FileName = schemaBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK) schemaBox.Text = dialog.FileName;
            }
        }

        private void OpenExport(object sender, EventArgs e)
        {
            string target = Directory.Exists(lastExport) ? lastExport : outputBox.Text;
            if (Directory.Exists(target)) Process.Start("explorer.exe", Quote(target));
        }

        private void OpenCandidates(object sender, EventArgs e)
        {
            OpenCandidatesFromExport(lastExport, true);
        }

        private void LoadParsedExport(object sender, EventArgs e)
        {
            using (FolderBrowserDialog dialog = new FolderBrowserDialog())
            {
                dialog.Description = "Select a parsed TF2 export folder containing manifest.json and frag_candidates.ndjson";
                if (Directory.Exists(lastExport)) dialog.SelectedPath = lastExport;
                else if (Directory.Exists(outputBox.Text)) dialog.SelectedPath = outputBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK)
                    OpenCandidatesFromExport(dialog.SelectedPath, true);
            }
        }

        private void OpenCandidatesFromExport(string exportDirectory, bool showErrors)
        {
            if (String.IsNullOrWhiteSpace(exportDirectory) || !Directory.Exists(exportDirectory))
            {
                if (showErrors)
                    MessageBox.Show(this, "Select an existing parsed export folder.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            string path = Path.Combine(exportDirectory, "frag_candidates.ndjson");
            string manifestPath = Path.Combine(exportDirectory, "manifest.json");
            if (!File.Exists(manifestPath) || !File.Exists(path))
            {
                if (showErrors)
                    MessageBox.Show(this,
                        "Select a parsed export folder containing both manifest.json and frag_candidates.ndjson. " +
                        "Run Parse STV demo first if this export has not been ranked yet.",
                        Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            if (candidateViewer != null) return;
            lastExport = exportDirectory;
            candidatesButton.Enabled = true;
            status.Text = "Loaded parsed export: " + exportDirectory;
            status.ForeColor = Color.LightGreen;
            Append("Loaded existing parsed export: " + exportDirectory + "\r\n");
            candidateViewer = new CandidateViewerForm(path);
            candidateViewer.BackRequested += ReturnToParser;
            candidateViewer.TopLevel = false;
            candidateViewer.FormBorderStyle = FormBorderStyle.None;
            candidateViewer.Dock = DockStyle.Fill;
            parserLayout.Visible = false;
            Controls.Add(candidateViewer);
            candidateViewer.BringToFront();
            candidateViewer.Show();
        }

        private void ReturnToParser(object sender, EventArgs e)
        {
            if (candidateViewer == null) return;
            CandidateViewerForm viewer = candidateViewer;
            candidateViewer = null;
            Controls.Remove(viewer);
            viewer.Dispose();
            parserLayout.Visible = true;
            parserLayout.BringToFront();
            progress.BringToFront();
        }

        private void OnDragEnter(object sender, DragEventArgs e)
        {
            if (e.Data.GetDataPresent(DataFormats.FileDrop)) e.Effect = DragDropEffects.Copy;
        }

        private void OnDragDrop(object sender, DragEventArgs e)
        {
            string[] files = e.Data.GetData(DataFormats.FileDrop) as string[];
            if (files == null || files.Length == 0) return;
            List<string> demos = new List<string>();
            foreach (string file in files)
                if (file.EndsWith(".dem", StringComparison.OrdinalIgnoreCase) && File.Exists(file)) demos.Add(file);
            if (demos.Count > 0) SetSelectedDemos(demos.ToArray());
        }

        private void SetSelectedDemos(string[] paths)
        {
            selectedDemos.Clear();
            HashSet<string> seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (string path in paths)
            {
                string fullPath = Path.GetFullPath(path);
                if (File.Exists(fullPath) && fullPath.EndsWith(".dem", StringComparison.OrdinalIgnoreCase) && seen.Add(fullPath))
                    selectedDemos.Add(fullPath);
            }
            demoSelectionDisplay = selectedDemos.Count == 1
                ? selectedDemos[0]
                : selectedDemos.Count + " demos selected: " + String.Join(" | ", selectedDemos.ConvertAll(Path.GetFileName).ToArray());
            demoBox.Text = demoSelectionDisplay;
            UpdateAutoDetectedItemSchema();
        }

        private void UpdateAutoDetectedItemSchema()
        {
            string current = schemaBox.Text.Trim();
            if (!String.IsNullOrEmpty(current) && !String.Equals(current, autoDetectedSchemaPath, StringComparison.OrdinalIgnoreCase)) return;
            string detected = DetectItemSchemaForSelectedDemos();
            schemaBox.Text = detected ?? "";
            autoDetectedSchemaPath = detected ?? "";
        }

        private string DetectItemSchemaForSelectedDemos()
        {
            string detected = null;
            foreach (string demo in selectedDemos)
            {
                string candidate = ItemSchemaBesideTfDemos(demo);
                if (String.IsNullOrEmpty(candidate)) return null;
                if (detected == null) detected = candidate;
                else if (!String.Equals(detected, candidate, StringComparison.OrdinalIgnoreCase)) return null;
            }
            return detected;
        }

        private static string ItemSchemaBesideTfDemos(string demoPath)
        {
            DirectoryInfo directory = new FileInfo(demoPath).Directory;
            while (directory != null)
            {
                if (String.Equals(directory.Name, "demos", StringComparison.OrdinalIgnoreCase) &&
                    directory.Parent != null && String.Equals(directory.Parent.Name, "tf", StringComparison.OrdinalIgnoreCase))
                {
                    string schema = Path.Combine(directory.Parent.FullName, "scripts", "items", "items_game.txt");
                    return File.Exists(schema) ? schema : null;
                }
                directory = directory.Parent;
            }
            return null;
        }

        private List<string> SelectedDemoPaths()
        {
            if (selectedDemos.Count > 0 && String.Equals(demoBox.Text, demoSelectionDisplay, StringComparison.Ordinal))
                return new List<string>(selectedDemos);
            List<string> result = new List<string>();
            string typed = demoBox.Text.Trim();
            if (File.Exists(typed) && typed.EndsWith(".dem", StringComparison.OrdinalIgnoreCase)) result.Add(Path.GetFullPath(typed));
            return result;
        }

        private static string SafeFileName(string value)
        {
            return BatchCandidateSupport.SafeName(value);
        }

        private void Append(string text)
        {
            if (log.InvokeRequired) { log.BeginInvoke(new Action<string>(Append), text); return; }
            log.AppendText(text);
            log.SelectionStart = log.TextLength;
            log.SelectionLength = 0;
            log.ScrollToCaret();
            string path = batchLogFilePath;
            if (!String.IsNullOrWhiteSpace(path))
            {
                lock (batchLogFileLock)
                {
                    try { File.AppendAllText(path, text, new UTF8Encoding(false)); }
                    catch { }
                }
            }
        }

        private static string Quote(string value) { return "\"" + value.Replace("\"", "\\\"") + "\""; }
    }

    internal sealed class ResourcePlan
    {
        private const ulong MiB = 1024UL * 1024UL;
        private const ulong GiB = 1024UL * 1024UL * 1024UL;

        public readonly int LogicalProcessors;
        public readonly ulong AvailableMemoryBytes;
        public readonly ulong ReservedMemoryBytes;
        public readonly ulong EstimatedParseWorkerBytes;
        public readonly ulong EstimatedAnalysisWorkerBytes;
        public readonly long DemoSize75thPercentileBytes;
        public readonly int ParseWorkers;
        public readonly int AnalysisWorkers;
        public readonly double HistoricalParseSecondsPerGiB;
        public readonly double HistoricalAnalysisSecondsPerGiB;

        private ResourcePlan(int logicalProcessors, ulong availableMemoryBytes, ulong reservedMemoryBytes,
            ulong estimatedParseWorkerBytes, ulong estimatedAnalysisWorkerBytes, long demoSize75thPercentileBytes,
            int parseWorkers, int analysisWorkers, double historicalParseSecondsPerGiB,
            double historicalAnalysisSecondsPerGiB)
        {
            LogicalProcessors = logicalProcessors;
            AvailableMemoryBytes = availableMemoryBytes;
            ReservedMemoryBytes = reservedMemoryBytes;
            EstimatedParseWorkerBytes = estimatedParseWorkerBytes;
            EstimatedAnalysisWorkerBytes = estimatedAnalysisWorkerBytes;
            DemoSize75thPercentileBytes = demoSize75thPercentileBytes;
            ParseWorkers = parseWorkers;
            AnalysisWorkers = analysisWorkers;
            HistoricalParseSecondsPerGiB = historicalParseSecondsPerGiB;
            HistoricalAnalysisSecondsPerGiB = historicalAnalysisSecondsPerGiB;
        }

        public static ResourcePlan Create(IList<string> demos)
        {
            return Create(demos, null);
        }

        public static ResourcePlan Create(IList<string> demos, BenchmarkHistory history)
        {
            int logicalProcessors = Math.Max(1, Environment.ProcessorCount);
            ulong availableMemory = SystemMemoryInfo.AvailablePhysicalMemoryBytes();
            long demoP75 = DemoSizePercentile(demos, 0.75);

            // export_all.rs currently reads the full .dem into memory. Leave extra room for
            // decoded parser/game state and JSON serialization buffers.
            ulong demoBytes = demoP75 > 0 ? (ulong)demoP75 : 256UL * MiB;
            ulong estimatedParseWorker = EstimateParseBytes(demoBytes);

            // The Python analyzer loads event/state histories and builds additional indexes.
            // Keep this conservative until it is ported to Rust and can report exact usage.
            ulong estimatedAnalysisWorker = EstimateAnalysisBytes(demoBytes);
            double historicalParseSecondsPerGiB = -1.0;
            double historicalAnalysisSecondsPerGiB = -1.0;
            if (history != null)
            {
                estimatedParseWorker = history.HistoricalParsePeakBytes(estimatedParseWorker);
                estimatedAnalysisWorker = history.HistoricalAnalysisPeakBytes(estimatedAnalysisWorker);
                historicalParseSecondsPerGiB = history.ParseSecondsPerGiB();
                historicalAnalysisSecondsPerGiB = history.AnalysisSecondsPerGiB();
            }

            int cpuReserve = logicalProcessors >= 12 ? 2 : (logicalProcessors >= 4 ? 1 : 0);
            int cpuBudgetWorkers = Math.Max(1, logicalProcessors - cpuReserve);

            ulong reservedMemory = 0;
            int parseMemoryWorkers = cpuBudgetWorkers;
            int analysisMemoryWorkers = cpuBudgetWorkers;
            if (availableMemory > 0)
            {
                reservedMemory = Math.Max(2UL * GiB, availableMemory / 5UL);
                if (reservedMemory >= availableMemory)
                    reservedMemory = availableMemory / 4UL;
                ulong usableMemory = availableMemory > reservedMemory ? availableMemory - reservedMemory : availableMemory;
                parseMemoryWorkers = Math.Max(1, SafeWorkerCount(usableMemory, estimatedParseWorker));
                analysisMemoryWorkers = Math.Max(1, SafeWorkerCount(usableMemory, estimatedAnalysisWorker));
            }

            // Full parse/export is also very write-heavy. This scalable I/O ceiling prevents a
            // high-core-count machine from launching dozens of giant NDJSON writers at once.
            int parseIoCeiling = Math.Max(2, (int)Math.Ceiling(Math.Sqrt(logicalProcessors) * 2.0));
            int parseWorkers = Math.Min(cpuBudgetWorkers, Math.Min(parseMemoryWorkers, parseIoCeiling));
            int analysisWorkers = Math.Min(cpuBudgetWorkers, analysisMemoryWorkers);
            if (history != null)
            {
                parseWorkers = history.RecommendParseWorkers(parseWorkers);
                analysisWorkers = history.RecommendAnalysisWorkers(analysisWorkers);
            }

            int demoCount = demos == null ? 0 : demos.Count;
            if (demoCount > 0)
            {
                parseWorkers = Math.Min(parseWorkers, demoCount);
                analysisWorkers = Math.Min(analysisWorkers, demoCount);
            }
            parseWorkers = Math.Max(1, parseWorkers);
            analysisWorkers = Math.Max(1, analysisWorkers);

            return new ResourcePlan(logicalProcessors, availableMemory, reservedMemory,
                estimatedParseWorker, estimatedAnalysisWorker, demoP75, parseWorkers, analysisWorkers,
                historicalParseSecondsPerGiB, historicalAnalysisSecondsPerGiB);
        }

        public string Describe()
        {
            StringBuilder text = new StringBuilder();
            text.AppendLine("AUTO RESOURCE PLAN");
            text.AppendLine("Logical CPU processors: " + LogicalProcessors);
            if (AvailableMemoryBytes > 0)
            {
                text.AppendLine("Available physical RAM: " + FormatBytes(AvailableMemoryBytes));
                text.AppendLine("RAM reserved for Windows/other applications: " + FormatBytes(ReservedMemoryBytes));
            }
            else
            {
                text.AppendLine("Available physical RAM: unavailable; CPU-only worker planning will be used.");
            }
            text.AppendLine("75th percentile demo size: " + FormatBytes(DemoSize75thPercentileBytes > 0 ? (ulong)DemoSize75thPercentileBytes : 0));
            text.AppendLine("Estimated RAM per parser worker: " + FormatBytes(EstimatedParseWorkerBytes));
            text.AppendLine("Estimated RAM per analyzer worker: " + FormatBytes(EstimatedAnalysisWorkerBytes));
            text.AppendLine("Automatic parser workers: " + ParseWorkers);
            text.AppendLine("Automatic analyzer workers: " + AnalysisWorkers);
            text.AppendLine("When enough successful same-machine benchmark runs exist at different worker counts, Auto can prefer the measured throughput winner without exceeding current CPU/RAM/I/O limits.");
            if (HistoricalParseSecondsPerGiB > 0.0)
                text.AppendLine("Historical parser time: " + HistoricalParseSecondsPerGiB.ToString("0.0") + " seconds per GiB of source demo per worker (median)." );
            if (HistoricalAnalysisSecondsPerGiB > 0.0)
                text.AppendLine("Historical analyzer time: " + HistoricalAnalysisSecondsPerGiB.ToString("0.0") + " seconds per GiB of parsed export per worker (median)." );
            text.AppendLine("Worker RAM estimates learn from recorded peak working-set benchmarks once enough samples exist. The parser-worker count also uses an I/O ceiling because full packet/state NDJSON exports are disk-write heavy. A live gate pauses new jobs when total CPU is already near 95% or free RAM falls below the reserved headroom.");
            return text.ToString();
        }

        public static ulong EstimateParseJobBytes(string demoPath)
        {
            return EstimateParseBytes(DemoSizeBytes(demoPath));
        }

        public static ulong EstimateAnalysisJobBytes(string demoPath)
        {
            return EstimateAnalysisBytes(DemoSizeBytes(demoPath));
        }

        private static ulong DemoSizeBytes(string demoPath)
        {
            try
            {
                if (File.Exists(demoPath)) return (ulong)Math.Max(0L, new FileInfo(demoPath).Length);
            }
            catch
            {
            }
            return 256UL * MiB;
        }

        private static ulong EstimateParseBytes(ulong demoBytes)
        {
            ulong estimate = 512UL * MiB + (demoBytes * 2UL);
            return Clamp(estimate, 768UL * MiB, 8UL * GiB);
        }

        private static ulong EstimateAnalysisBytes(ulong demoBytes)
        {
            ulong estimate = 768UL * MiB + demoBytes;
            return Clamp(estimate, 1UL * GiB, 8UL * GiB);
        }

        private static long DemoSizePercentile(IList<string> demos, double percentile)
        {
            List<long> sizes = new List<long>();
            if (demos != null)
            {
                foreach (string demo in demos)
                {
                    try
                    {
                        if (File.Exists(demo)) sizes.Add(new FileInfo(demo).Length);
                    }
                    catch
                    {
                    }
                }
            }
            if (sizes.Count == 0) return 0;
            sizes.Sort();
            int index = (int)Math.Ceiling((sizes.Count - 1) * percentile);
            if (index < 0) index = 0;
            if (index >= sizes.Count) index = sizes.Count - 1;
            return sizes[index];
        }

        private static int SafeWorkerCount(ulong usableMemory, ulong perWorker)
        {
            if (perWorker == 0) return 1;
            ulong count = usableMemory / perWorker;
            if (count == 0) return 1;
            if (count > Int32.MaxValue) return Int32.MaxValue;
            return (int)count;
        }

        private static ulong Clamp(ulong value, ulong minimum, ulong maximum)
        {
            if (value < minimum) return minimum;
            if (value > maximum) return maximum;
            return value;
        }

        private static string FormatBytes(ulong bytes)
        {
            if (bytes >= GiB) return (bytes / (double)GiB).ToString("0.0") + " GB";
            if (bytes >= MiB) return (bytes / (double)MiB).ToString("0") + " MB";
            if (bytes >= 1024UL) return (bytes / 1024.0).ToString("0") + " KB";
            return bytes + " B";
        }
    }

    internal sealed class AdaptiveResourceGate : IDisposable
    {
        private const double CpuUsageCeilingPercent = 95.0;
        private readonly object sync = new object();
        private readonly ulong reservedMemoryBytes;
        private int activeJobs;
        private bool disposed;

        public AdaptiveResourceGate(ulong reservedMemoryBytes)
        {
            this.reservedMemoryBytes = reservedMemoryBytes;
        }

        public async Task EnterAsync(ulong estimatedJobBytes, CancellationToken cancellationToken)
        {
            while (true)
            {
                cancellationToken.ThrowIfCancellationRequested();
                bool allow = false;
                lock (sync)
                {
                    if (disposed) throw new ObjectDisposedException("AdaptiveResourceGate");
                    ulong available = SystemMemoryInfo.AvailablePhysicalMemoryBytes();
                    double cpuUsage = SystemCpuInfo.CurrentUsagePercent();
                    bool memoryOkay = available == 0 || available >= reservedMemoryBytes + estimatedJobBytes;
                    bool cpuOkay = cpuUsage < 0.0 || cpuUsage < CpuUsageCeilingPercent;

                    // Always allow one job so a transient low-memory/high-CPU reading cannot
                    // deadlock the whole batch. Additional jobs start only while live RAM and
                    // live total-CPU pressure remain inside the automatic resource budget.
                    if (activeJobs == 0 || (memoryOkay && cpuOkay))
                    {
                        activeJobs++;
                        allow = true;
                    }
                }

                if (allow) return;
                await Task.Delay(500, cancellationToken);
            }
        }

        public void Exit()
        {
            lock (sync)
            {
                if (activeJobs > 0) activeJobs--;
            }
        }

        public void Dispose()
        {
            lock (sync)
            {
                disposed = true;
            }
        }
    }

    internal static class SystemCpuInfo
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct NativeFileTime
        {
            public uint LowDateTime;
            public uint HighDateTime;
        }

        private static readonly object sync = new object();
        private static ulong previousIdle;
        private static ulong previousKernel;
        private static ulong previousUser;
        private static long previousSampleUtcTicks;
        private static double cachedUsage = -1.0;

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetSystemTimes(out NativeFileTime idleTime, out NativeFileTime kernelTime, out NativeFileTime userTime);

        public static double CurrentUsagePercent()
        {
            lock (sync)
            {
                try
                {
                    long now = DateTime.UtcNow.Ticks;
                    if (previousSampleUtcTicks != 0 && now - previousSampleUtcTicks < TimeSpan.TicksPerMillisecond * 250L)
                        return cachedUsage;

                    NativeFileTime idleTime;
                    NativeFileTime kernelTime;
                    NativeFileTime userTime;
                    if (!GetSystemTimes(out idleTime, out kernelTime, out userTime)) return -1.0;

                    ulong idle = ToUInt64(idleTime);
                    ulong kernel = ToUInt64(kernelTime);
                    ulong user = ToUInt64(userTime);

                    if (previousSampleUtcTicks == 0)
                    {
                        previousIdle = idle;
                        previousKernel = kernel;
                        previousUser = user;
                        previousSampleUtcTicks = now;
                        cachedUsage = -1.0;
                        return cachedUsage;
                    }

                    ulong idleDelta = idle >= previousIdle ? idle - previousIdle : 0;
                    ulong kernelDelta = kernel >= previousKernel ? kernel - previousKernel : 0;
                    ulong userDelta = user >= previousUser ? user - previousUser : 0;
                    ulong total = kernelDelta + userDelta;

                    previousIdle = idle;
                    previousKernel = kernel;
                    previousUser = user;
                    previousSampleUtcTicks = now;

                    if (total == 0) return cachedUsage;
                    ulong busy = total > idleDelta ? total - idleDelta : 0;
                    cachedUsage = Math.Max(0.0, Math.Min(100.0, (busy * 100.0) / total));
                    return cachedUsage;
                }
                catch
                {
                    return -1.0;
                }
            }
        }

        private static ulong ToUInt64(NativeFileTime time)
        {
            return ((ulong)time.HighDateTime << 32) | time.LowDateTime;
        }
    }

    internal static class SystemMemoryInfo
    {
        [StructLayout(LayoutKind.Sequential)]
        private struct MemoryStatusEx
        {
            public uint Length;
            public uint MemoryLoad;
            public ulong TotalPhysical;
            public ulong AvailablePhysical;
            public ulong TotalPageFile;
            public ulong AvailablePageFile;
            public ulong TotalVirtual;
            public ulong AvailableVirtual;
            public ulong AvailableExtendedVirtual;
        }

        [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GlobalMemoryStatusEx(ref MemoryStatusEx buffer);

        public static ulong AvailablePhysicalMemoryBytes()
        {
            try
            {
                MemoryStatusEx status = new MemoryStatusEx();
                status.Length = (uint)Marshal.SizeOf(typeof(MemoryStatusEx));
                if (GlobalMemoryStatusEx(ref status)) return status.AvailablePhysical;
            }
            catch
            {
            }
            return 0;
        }

        public static ulong TotalPhysicalMemoryBytes()
        {
            try
            {
                MemoryStatusEx status = new MemoryStatusEx();
                status.Length = (uint)Marshal.SizeOf(typeof(MemoryStatusEx));
                if (GlobalMemoryStatusEx(ref status)) return status.TotalPhysical;
            }
            catch
            {
            }
            return 0;
        }
    }

    internal sealed class CandidateViewerForm : Form
    {
        private const string PlaybackTempPrefix = "tf2fragdemohelper_temp_";
        private readonly string candidatesPath;
        private readonly DataGridView grid = new DataGridView();
        private readonly TextBox details = new TextBox();
        private readonly Label summary = new Label();
        private readonly TextBox filterBox = new TextBox();
        private readonly NumericUpDown minimumScore = new NumericUpDown();
        private readonly NumericUpDown leadInSeconds = new NumericUpDown();
        private readonly NumericUpDown outroSeconds = new NumericUpDown();
        private readonly ComboBox recordingFps = new ComboBox();
        private readonly NumericUpDown jpgQuality = new NumericUpDown();
        private readonly ComboBox recordingOutput = new ComboBox();
        private readonly Label recordingEstimate = new Label();
        private readonly Button inlinePreviewButton = GreenButton("Preview Selected Clip in TF2", 190);
        private readonly Button selectAllButton = GreenButton("Select all visible", 135);
        private readonly Button recordButton = GreenButton("Record selected with HLAE", 205);
        private readonly Button backButton = GreenButton("Back to parser", 165);
        private readonly List<CandidateRecord> records = new List<CandidateRecord>();
        private string demoPath;
        private string tf2Executable;
        private bool detailsScrollPending;
        private int clickedSelectedRow = -1;
        private bool applyingFilter;

        public event EventHandler BackRequested;

        private static Button GreenButton(string text, int width)
        {
            Button button = new Button();
            button.Text = text;
            button.Width = width;
            button.Height = 32;
            button.FlatStyle = FlatStyle.Flat;
            button.BackColor = Color.FromArgb(44, 130, 82);
            button.ForeColor = Color.White;
            button.FlatAppearance.BorderColor = Color.FromArgb(64, 160, 103);
            return button;
        }

        private static string Quote(string value)
        {
            return "\"" + value.Replace("\"", "\\\"") + "\"";
        }

        public CandidateViewerForm(string path)
        {
            candidatesPath = path;
            Text = "TF2 Frag Candidates";
            StartPosition = FormStartPosition.CenterScreen;
            MinimumSize = new Size(1400, 760);
            Size = new Size(1480, 860);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;
            BuildPage();
            LoadCandidates();
            HlaeBatchRecorder.RecordedClipVerified += OnRecordedClipVerified;
            FormClosed += delegate { HlaeBatchRecorder.RecordedClipVerified -= OnRecordedClipVerified; };
            Shown += delegate
            {
                BeginInvoke(new MethodInvoker(delegate
                {
                    ClearCandidateSelection();
                    ScrollDetailsToBottom();
                }));
            };
        }

        private void BuildPage()
        {
            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.Padding = new Padding(14);
            layout.ColumnCount = 1;
            layout.RowCount = 2;
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 188));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            Controls.Add(layout);

            TableLayoutPanel filters = new TableLayoutPanel();
            filters.Dock = DockStyle.Fill;
            filters.ColumnCount = 1;
            filters.RowCount = 5;
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 40));
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 36));
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 40));
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 40));
            FlowLayoutPanel heading = new FlowLayoutPanel();
            heading.Dock = DockStyle.Fill;
            heading.FlowDirection = FlowDirection.LeftToRight;
            heading.WrapContents = false;
            summary.AutoSize = true;
            summary.Margin = new Padding(3, 5, 3, 0);
            summary.ForeColor = Color.Silver;
            heading.Controls.Add(summary);
            filters.Controls.Add(heading);

            FlowLayoutPanel filterControls = new FlowLayoutPanel();
            filterControls.Dock = DockStyle.Fill;
            filterControls.FlowDirection = FlowDirection.LeftToRight;
            filterControls.WrapContents = false;
            Label filterLabel = new Label();
            filterLabel.Text = "Filter (field-specific: +class:demoman -map:cp_steel +mode:rgl_6v6 +type:stv +recorded:true; combine multiple filters)";
            filterLabel.AutoSize = true;
            filterLabel.Margin = new Padding(3, 9, 4, 2);
            filterControls.Controls.Add(filterLabel);
            filterBox.Width = 360;
            filterBox.Margin = new Padding(0, 5, 14, 2);
            filterBox.TextChanged += delegate { ApplyFilter(); };
            filterControls.Controls.Add(filterBox);
            Label minimumLabel = new Label();
            minimumLabel.Text = "Minimum score";
            minimumLabel.AutoSize = true;
            minimumLabel.Margin = new Padding(3, 9, 4, 2);
            filterControls.Controls.Add(minimumLabel);
            minimumScore.Width = 70;
            minimumScore.Maximum = 1000;
            minimumScore.Margin = new Padding(0, 5, 2, 2);
            minimumScore.ValueChanged += delegate { ApplyFilter(); };
            filterControls.Controls.Add(minimumScore);
            backButton.Margin = new Padding(16, 3, 2, 2);
            backButton.Click += delegate
            {
                EventHandler handler = BackRequested;
                if (handler != null) handler(this, EventArgs.Empty);
            };
            filterControls.Controls.Add(backButton);

            filters.Controls.Add(filterControls, 0, 1);

            FlowLayoutPanel playbackControls = new FlowLayoutPanel();
            playbackControls.Dock = DockStyle.Fill;
            playbackControls.FlowDirection = FlowDirection.LeftToRight;
            playbackControls.WrapContents = false;
            Label leadLabel = new Label();
            leadLabel.Text = "Seconds before first event";
            leadLabel.AutoSize = true;
            leadLabel.Margin = new Padding(14, 9, 4, 2);
            playbackControls.Controls.Add(leadLabel);
            leadInSeconds.Width = 58;
            leadInSeconds.Minimum = 0;
            leadInSeconds.Maximum = 60;
            leadInSeconds.Value = 8;
            leadInSeconds.Increment = 1;
            leadInSeconds.Margin = new Padding(0, 5, 10, 2);
            leadInSeconds.ValueChanged += delegate { UpdateRecordingEstimate(); };
            playbackControls.Controls.Add(leadInSeconds);
            Label outroLabel = new Label();
            outroLabel.Text = "Seconds after last event";
            outroLabel.AutoSize = true;
            outroLabel.Margin = new Padding(4, 9, 4, 2);
            playbackControls.Controls.Add(outroLabel);
            outroSeconds.Width = 52;
            outroSeconds.Minimum = 0;
            outroSeconds.Maximum = 60;
            outroSeconds.Value = 3;
            outroSeconds.Margin = new Padding(0, 5, 10, 2);
            outroSeconds.ValueChanged += delegate { UpdateRecordingEstimate(); };
            playbackControls.Controls.Add(outroSeconds);
            Label fpsLabel = new Label();
            fpsLabel.Text = "Capture FPS";
            fpsLabel.AutoSize = true;
            fpsLabel.Margin = new Padding(4, 9, 4, 2);
            playbackControls.Controls.Add(fpsLabel);
            recordingFps.DropDownStyle = ComboBoxStyle.DropDownList;
            recordingFps.Width = 72;
            recordingFps.Items.Add("60");
            recordingFps.Items.Add("120");
            recordingFps.Items.Add("240");
            recordingFps.Items.Add("480");
            recordingFps.SelectedIndex = 1;
            recordingFps.Margin = new Padding(0, 5, 10, 2);
            recordingFps.SelectedIndexChanged += delegate { UpdateRecordingEstimate(); };
            playbackControls.Controls.Add(recordingFps);
            filters.Controls.Add(playbackControls, 0, 2);

            FlowLayoutPanel recordingControls = new FlowLayoutPanel();
            recordingControls.Dock = DockStyle.Fill;
            recordingControls.FlowDirection = FlowDirection.LeftToRight;
            recordingControls.WrapContents = false;
            Label outputLabel = new Label();
            outputLabel.Text = "Recording output";
            outputLabel.AutoSize = true;
            outputLabel.Margin = new Padding(14, 9, 4, 2);
            recordingControls.Controls.Add(outputLabel);
            recordingOutput.DropDownStyle = ComboBoxStyle.DropDownList;
            recordingOutput.Width = 180;
            recordingOutput.Items.Add("TGA Image Sequence");
            recordingOutput.Items.Add("JPG Image Sequence");
            recordingOutput.Items.Add("MP4 - Standard");
            recordingOutput.Items.Add("MP4 - Compatible");
            recordingOutput.Items.Add("MP4 - Lossless");
            recordingOutput.Items.Add("AVI - Raw");
            recordingOutput.SelectedIndex = 2;
            recordingOutput.Margin = new Padding(0, 5, 10, 2);
            recordingOutput.SelectedIndexChanged += delegate { UpdateOutputDescription(); };
            recordingControls.Controls.Add(recordingOutput);
            Label jpgLabel = new Label();
            jpgLabel.Text = "JPG quality";
            jpgLabel.AutoSize = true;
            jpgLabel.Margin = new Padding(3, 9, 4, 2);
            recordingControls.Controls.Add(jpgLabel);
            jpgQuality.Width = 50;
            jpgQuality.Minimum = 1;
            jpgQuality.Maximum = 100;
            jpgQuality.Value = 90;
            jpgQuality.Margin = new Padding(0, 5, 14, 2);
            jpgQuality.ValueChanged += delegate { UpdateRecordingEstimate(); };
            recordingControls.Controls.Add(jpgQuality);
            selectAllButton.Margin = new Padding(4, 3, 2, 2);
            selectAllButton.Click += delegate { SelectAllVisibleCandidates(); };
            recordingControls.Controls.Add(selectAllButton);
            recordButton.Margin = new Padding(4, 3, 2, 2);
            recordButton.Click += delegate { RecordSelectedCandidates(); };
            recordingControls.Controls.Add(recordButton);
            inlinePreviewButton.Margin = new Padding(4, 3, 2, 2);
            inlinePreviewButton.Visible = false;
            inlinePreviewButton.Click += delegate { LaunchSelectedCandidate(); };
            recordingControls.Controls.Add(inlinePreviewButton);
            filters.Controls.Add(recordingControls, 0, 3);
            recordingEstimate.Dock = DockStyle.Fill;
            recordingEstimate.AutoEllipsis = true;
            recordingEstimate.ForeColor = Color.LightSkyBlue;
            recordingEstimate.TextAlign = ContentAlignment.MiddleLeft;
            recordingEstimate.Margin = new Padding(14, 0, 3, 0);
            filters.Controls.Add(recordingEstimate, 0, 4);
            layout.Controls.Add(filters, 0, 0);

            SplitContainer split = new SplitContainer();
            split.Dock = DockStyle.Fill;
            split.Orientation = Orientation.Horizontal;
            split.SplitterDistance = 330;
            layout.Controls.Add(split, 0, 1);

            grid.Dock = DockStyle.Fill;
            grid.ReadOnly = true;
            grid.AllowUserToAddRows = false;
            grid.AllowUserToDeleteRows = false;
            grid.AllowUserToResizeRows = false;
            grid.RowHeadersVisible = false;
            grid.SelectionMode = DataGridViewSelectionMode.FullRowSelect;
            grid.MultiSelect = true;
            grid.AutoGenerateColumns = false;
            grid.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.None;
            grid.AllowUserToOrderColumns = true;
            grid.AllowUserToResizeColumns = true;
            grid.BackgroundColor = Color.FromArgb(17, 18, 20);
            grid.GridColor = Color.FromArgb(62, 66, 72);
            grid.DefaultCellStyle.BackColor = Color.FromArgb(24, 26, 29);
            grid.DefaultCellStyle.ForeColor = Color.Gainsboro;
            grid.DefaultCellStyle.SelectionBackColor = Color.FromArgb(46, 105, 76);
            grid.DefaultCellStyle.SelectionForeColor = Color.White;
            grid.ColumnHeadersDefaultCellStyle.BackColor = Color.FromArgb(44, 48, 54);
            grid.ColumnHeadersDefaultCellStyle.ForeColor = Color.White;
            grid.EnableHeadersVisualStyles = false;
            AddColumn("#", 42);
            AddColumn("Score", 60);
            AddColumn("Kills", 52);
            AddColumn("Attacker", 88);
            AddColumn("Class", 95);
            AddColumn("Team", 72);
            AddColumn("Demo", 170);
            AddColumn("Map", 170);
            AddColumn("Mode", 165);
            AddColumn("Demo Type", 90);
            AddColumn("Recorded", 76);
            AddColumn("Exact kill-event ticks", 175);
            AddColumn("Tags", 400);
            grid.SelectionChanged += ShowSelectedCandidate;
            grid.CellMouseDown += RememberSelectedRowClick;
            grid.CellClick += ToggleClickedSelectedRow;
            split.Panel1.Controls.Add(grid);

            details.Dock = DockStyle.Fill;
            details.Multiline = true;
            details.ReadOnly = true;
            details.ScrollBars = ScrollBars.Both;
            details.WordWrap = false;
            details.BackColor = Color.FromArgb(17, 18, 20);
            details.ForeColor = Color.FromArgb(218, 224, 230);
            details.Font = new Font("Consolas", 10F);
            details.TextChanged += delegate { ScrollDetailsToBottom(); };
            details.HandleCreated += delegate { ScrollDetailsToBottom(); };
            split.Panel2.Controls.Add(details);
            UpdateOutputDescription();
            UpdateCandidateActionAvailability();
        }

        private void AddColumn(string name, int width)
        {
            DataGridViewTextBoxColumn column = new DataGridViewTextBoxColumn();
            column.Name = name;
            column.HeaderText = name;
            column.Width = width;
            if (String.Equals(name, "Recorded", StringComparison.OrdinalIgnoreCase))
            {
                column.DefaultCellStyle.Alignment = DataGridViewContentAlignment.MiddleCenter;
                column.HeaderCell.Style.Alignment = DataGridViewContentAlignment.MiddleCenter;
            }
            column.SortMode = DataGridViewColumnSortMode.NotSortable;
            grid.Columns.Add(column);
        }

        private void LoadCandidates()
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            int rank = 0;
            try
            {
                foreach (string line in File.ReadLines(candidatesPath))
                {
                    if (String.IsNullOrWhiteSpace(line)) continue;
                    IDictionary candidate = serializer.DeserializeObject(line) as IDictionary;
                    if (candidate == null) continue;
                    rank++;
                    records.Add(new CandidateRecord(rank, candidate, line));
                }
            }
            catch (Exception error)
            {
                MessageBox.Show(this, "Could not read frag_candidates.ndjson:\r\n" + error.Message, Text, MessageBoxButtons.OK, MessageBoxIcon.Error);
                Close();
                return;
            }
            LoadDemoPath(serializer);
            ApplyFilter();
        }

        private void LoadDemoPath(JavaScriptSerializer serializer)
        {
            string manifestPath = Path.Combine(Path.GetDirectoryName(candidatesPath), "manifest.json");
            if (!File.Exists(manifestPath)) return;
            try
            {
                IDictionary manifest = serializer.DeserializeObject(File.ReadAllText(manifestPath)) as IDictionary;
                string source = TextValue(manifest, "source_demo");
                if (!String.IsNullOrEmpty(source) && File.Exists(source)) demoPath = source;
                FindTf2ExecutableNearDemo(demoPath);
            }
            catch { demoPath = null; }
        }

        private void FindTf2ExecutableNearDemo(string sourceDemoPath)
        {
            if (String.IsNullOrEmpty(sourceDemoPath) || !File.Exists(sourceDemoPath)) return;
            DirectoryInfo directory = new FileInfo(sourceDemoPath).Directory;
            DirectoryInfo tfDirectory = null;
            for (int depth = 0; directory != null && depth < 8; depth++, directory = directory.Parent)
            {
                if (String.Equals(directory.Name, "tf", StringComparison.OrdinalIgnoreCase))
                {
                    tfDirectory = directory;
                    break;
                }
            }
            if (tfDirectory == null || tfDirectory.Parent == null) return;
            string demosRoot = Path.GetFullPath(Path.Combine(tfDirectory.FullName, "demos")).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            string demoDirectory = Path.GetFullPath(Path.GetDirectoryName(sourceDemoPath)).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
            if (!demoDirectory.StartsWith(demosRoot, StringComparison.OrdinalIgnoreCase)) return;
            foreach (string executableName in new string[] { "tf_win64.exe", "tf.exe" })
            {
                string candidate = Path.Combine(tfDirectory.Parent.FullName, executableName);
                if (File.Exists(candidate))
                {
                    tf2Executable = candidate;
                    return;
                }
            }
        }

        private void ApplyFilter()
        {
            if (grid.Columns.Count == 0) return;
            HashSet<string> selectedCandidateKeys = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (DataGridViewRow selectedRow in grid.SelectedRows)
            {
                IDictionary selectedCandidate = selectedRow.Tag as IDictionary;
                if (selectedCandidate != null) selectedCandidateKeys.Add(CandidateSelectionKey(selectedCandidate));
            }
            string filter = filterBox.Text.Trim();
            decimal requiredScore = minimumScore.Value;
            applyingFilter = true;
            grid.Rows.Clear();
            int visible = 0;
            string exportDirectory = Path.GetDirectoryName(candidatesPath);
            foreach (CandidateRecord record in records)
            {
                if (record.Score < requiredScore) continue;
                IDictionary candidate = record.Candidate;
                string mapName = BatchCandidateSupport.CandidateMapName(candidate, exportDirectory);
                if (!CandidateMatchesFilter(record, candidate, mapName, demoPath, filter)) continue;
                IDictionary metrics = Map(candidate, "metrics");
                IList killTicks = List(candidate, "point_of_kill_ticks");
                int row = grid.Rows.Add(
                    record.Rank,
                    TextValue(candidate, "overall_score"),
                    TextValue(metrics, "kills"),
                    "#" + TextValue(candidate, "attacker_user_id"),
                    DisplayValue(candidate, "attacker_class"),
                    DisplayValue(candidate, "attacker_team"),
                    BatchCandidateSupport.CandidateDemoName(candidate, demoPath),
                    mapName,
                    CandidateModeLabel(candidate),
                    CandidateDemoTypeLabel(candidate),
                    HlaeBatchRecorder.IsCandidateAlreadyRecorded(candidate, demoPath) ? "Recorded" : "",
                    JoinValues(killTicks),
                    JoinCandidateTags(Value(candidate, "tags")));
                grid.Rows[row].Tag = candidate;
                visible++;
            }
            grid.ClearSelection();
            grid.CurrentCell = null;
            if (selectedCandidateKeys.Count > 0)
            {
                foreach (DataGridViewRow row in grid.Rows)
                {
                    IDictionary candidate = row.Tag as IDictionary;
                    if (candidate != null && selectedCandidateKeys.Contains(CandidateSelectionKey(candidate))) row.Selected = true;
                }
            }
            applyingFilter = false;
            FitCandidateColumnsToContent();
            summary.Text = visible + " of " + records.Count + " ranked candidates. Select one or more rows before recording.";
            if (grid.Rows.Count == 0)
                details.Text = records.Count == 0 ? "No candidates were produced for this demo." : "No candidates match the current filter.";
            else
                details.Clear();
            UpdateCandidateActionAvailability();
        }

        private void OnRecordedClipVerified(object sender, EventArgs e)
        {
            if (IsDisposed || Disposing) return;
            if (InvokeRequired)
            {
                try { BeginInvoke(new MethodInvoker(RefreshRecordedCandidateCells)); }
                catch { }
                return;
            }
            RefreshRecordedCandidateCells();
        }

        private void RefreshRecordedCandidateCells()
        {
            if (IsDisposed || Disposing) return;
            ApplyFilter();
        }

        private string CandidateSelectionKey(IDictionary candidate)
        {
            return BatchCandidateSupport.CandidateDemoPath(candidate, demoPath) + "|" +
                TextValue(candidate, "candidate_id") + "|" +
                ClipTick(candidate, "clip_start_tick", "start_tick") + "|" +
                ClipTick(candidate, "clip_end_tick", "end_tick");
        }

        private static string CandidateMode(IDictionary candidate)
        {
            return TextValue(Map(candidate, "demo_context"), "mode");
        }

        private static string CandidateModeLabel(IDictionary candidate)
        {
            string label = TextValue(Map(candidate, "demo_context"), "mode_label");
            return String.IsNullOrEmpty(label) ? "Unknown / Mixed" : label;
        }

        private static string CandidateDemoType(IDictionary candidate)
        {
            return TextValue(Map(candidate, "demo_context"), "capture_type");
        }

        private static string CandidateDemoTypeLabel(IDictionary candidate)
        {
            string captureType = CandidateDemoType(candidate);
            if (String.Equals(captureType, "stv", StringComparison.OrdinalIgnoreCase)) return "STV";
            if (String.Equals(captureType, "pov", StringComparison.OrdinalIgnoreCase)) return "POV";
            return "Unknown";
        }

        private static bool CandidateMatchesFilter(CandidateRecord record, IDictionary candidate, string mapName, string fallbackDemoPath, string filter)
        {
            if (String.IsNullOrWhiteSpace(filter)) return true;
            string demoName = BatchCandidateSupport.CandidateDemoName(candidate, fallbackDemoPath);
            string demoPathValue = BatchCandidateSupport.CandidateDemoPath(candidate, fallbackDemoPath);
            bool recorded = HlaeBatchRecorder.IsCandidateAlreadyRecorded(candidate, fallbackDemoPath);
            string searchText = String.Join(" ", new string[]
            {
                record.SearchText ?? "",
                mapName ?? "",
                demoName ?? "",
                demoPathValue ?? "",
                CandidateMode(candidate) ?? "",
                CandidateModeLabel(candidate) ?? "",
                CandidateDemoType(candidate) ?? "",
                CandidateDemoTypeLabel(candidate) ?? "",
                TextValue(candidate, "attacker_class"),
                TextValue(candidate, "attacker_team"),
                TextValue(candidate, "attacker_user_id"),
                JoinCandidateTags(Value(candidate, "tags")),
                recorded ? "recorded" : "unrecorded"
            });
            CandidateFilterExpression expression = CandidateFilterExpression.Parse(filter);
            return expression.Matches(
                delegate(string field, string value)
                {
                    return CandidateFieldMatches(candidate, mapName, fallbackDemoPath, field, value, searchText, recorded);
                },
                delegate(string value) { return ContainsFilterValue(searchText, value); });
        }

        private static bool CandidateFieldMatches(IDictionary candidate, string mapName, string fallbackDemoPath, string field, string value, string searchText, bool recorded)
        {
            if (String.Equals(field, "recorded", StringComparison.OrdinalIgnoreCase))
            {
                string wanted = (value ?? "").Trim().ToLowerInvariant();
                if (wanted == "true" || wanted == "yes" || wanted == "1" || wanted == "recorded") return recorded;
                if (wanted == "false" || wanted == "no" || wanted == "0" || wanted == "unrecorded") return !recorded;
                return false;
            }

            if (String.Equals(field, "map", StringComparison.OrdinalIgnoreCase))
                return ContainsFilterValue(mapName, value);

            if (String.Equals(field, "mode", StringComparison.OrdinalIgnoreCase))
                return ContainsFilterValue(CandidateMode(candidate), value) || ContainsFilterValue(CandidateModeLabel(candidate), value);

            if (String.Equals(field, "type", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "demo_type", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "capture", StringComparison.OrdinalIgnoreCase))
                return ContainsFilterValue(CandidateDemoType(candidate), value) || ContainsFilterValue(CandidateDemoTypeLabel(candidate), value);

            if (String.Equals(field, "class", StringComparison.OrdinalIgnoreCase))
            {
                if (ClassValueMatches(TextValue(candidate, "attacker_class"), value)) return true;
                IDictionary attacker = Map(candidate, "attacker");
                if (ClassValueMatches(TextValue(attacker, "class"), value) ||
                    ClassValueMatches(TextValue(attacker, "class_id"), value) ||
                    ClassValueMatches(TextValue(attacker, "player_class"), value)) return true;
                IList kills = List(candidate, "kills");
                for (int index = 0; index < kills.Count; index++)
                {
                    IDictionary kill = kills[index] as IDictionary;
                    if (kill == null) continue;
                    if (ClassValueMatches(TextValue(kill, "attacker_class"), value)) return true;
                    IDictionary killAttacker = Map(kill, "attacker");
                    if (ClassValueMatches(TextValue(killAttacker, "class"), value) ||
                        ClassValueMatches(TextValue(killAttacker, "class_id"), value)) return true;
                }
                return false;
            }

            if (String.Equals(field, "team", StringComparison.OrdinalIgnoreCase))
            {
                if (TeamValueMatches(TextValue(candidate, "attacker_team"), value)) return true;
                IDictionary attacker = Map(candidate, "attacker");
                if (TeamValueMatches(TextValue(attacker, "team"), value) ||
                    TeamValueMatches(TextValue(attacker, "team_id"), value)) return true;
                IList kills = List(candidate, "kills");
                for (int index = 0; index < kills.Count; index++)
                {
                    IDictionary kill = kills[index] as IDictionary;
                    if (kill == null) continue;
                    if (TeamValueMatches(TextValue(kill, "attacker_team"), value)) return true;
                    IDictionary killAttacker = Map(kill, "attacker");
                    if (TeamValueMatches(TextValue(killAttacker, "team"), value) ||
                        TeamValueMatches(TextValue(killAttacker, "team_id"), value)) return true;
                }
                return false;
            }

            if (String.Equals(field, "demo", StringComparison.OrdinalIgnoreCase))
            {
                string name = BatchCandidateSupport.CandidateDemoName(candidate, fallbackDemoPath);
                string path = BatchCandidateSupport.CandidateDemoPath(candidate, fallbackDemoPath);
                return ContainsFilterValue(name, value) || ContainsFilterValue(path, value);
            }

            if (String.Equals(field, "weapon", StringComparison.OrdinalIgnoreCase))
            {
                if (ContainsFilterValue(TextValue(candidate, "weapon"), value) ||
                    ContainsFilterValue(TextValue(candidate, "weapon_logclassname"), value)) return true;
                IList kills = List(candidate, "kills");
                for (int index = 0; index < kills.Count; index++)
                {
                    IDictionary kill = kills[index] as IDictionary;
                    if (kill == null) continue;
                    if (ContainsFilterValue(TextValue(kill, "weapon"), value)) return true;
                    if (ContainsFilterValue(TextValue(kill, "weapon_logclassname"), value)) return true;
                    if (ContainsFilterValue(TextValue(kill, "weapon_name"), value)) return true;
                }
                return false;
            }

            if (String.Equals(field, "player", StringComparison.OrdinalIgnoreCase))
            {
                string wanted = value.Trim().TrimStart('#');
                if (PlayerValueMatches(candidate, wanted)) return true;
                IList kills = List(candidate, "kills");
                for (int index = 0; index < kills.Count; index++)
                {
                    IDictionary kill = kills[index] as IDictionary;
                    if (kill != null && PlayerValueMatches(kill, wanted)) return true;
                }
                return false;
            }

            if (String.Equals(field, "tag", StringComparison.OrdinalIgnoreCase))
            {
                IList tags = Value(candidate, "tags") as IList;
                if (tags != null)
                {
                    foreach (object tag in tags)
                    {
                        string raw = Convert.ToString(tag);
                        if (ContainsFilterValue(raw, value) || ContainsFilterValue(CandidateTagName(raw), value)) return true;
                    }
                }
                return false;
            }

            if (String.Equals(field, "text", StringComparison.OrdinalIgnoreCase))
                return ContainsFilterValue(searchText, value);

            return false;
        }

        private static bool PlayerValueMatches(IDictionary values, string wanted)
        {
            if (values == null || String.IsNullOrWhiteSpace(wanted)) return false;
            string id = TextValue(values, "attacker_user_id").TrimStart('#');
            if (String.Equals(id, wanted, StringComparison.OrdinalIgnoreCase)) return true;
            foreach (string key in new string[] { "attacker_name", "attacker_steamid", "attacker_steam_id", "attacker_steamid64" })
                if (ContainsFilterValue(TextValue(values, key), wanted)) return true;

            IDictionary attacker = Map(values, "attacker");
            foreach (string key in new string[] { "user_id", "userid", "name", "steamid", "steam_id", "steamid64" })
            {
                string nested = TextValue(attacker, key);
                if ((key == "user_id" || key == "userid") && String.Equals(nested.TrimStart('#'), wanted, StringComparison.OrdinalIgnoreCase)) return true;
                if (key != "user_id" && key != "userid" && ContainsFilterValue(nested, wanted)) return true;
            }
            return false;
        }

        private static bool ClassValueMatches(string actual, string wanted)
        {
            if (ContainsFilterValue(actual, wanted)) return true;
            string normalizedActual = NormalizeClassValue(actual);
            string normalizedWanted = NormalizeClassValue(wanted);
            return normalizedActual.Length > 0 && normalizedWanted.Length > 0 &&
                (ContainsFilterValue(normalizedActual, normalizedWanted) || ContainsFilterValue(normalizedWanted, normalizedActual));
        }

        private static string NormalizeClassValue(string value)
        {
            string text = (value ?? "").Trim().ToLowerInvariant();
            int numeric;
            if (Int32.TryParse(text, out numeric))
            {
                switch (numeric)
                {
                    case 1: return "scout";
                    case 2: return "sniper";
                    case 3: return "soldier";
                    case 4: return "demoman";
                    case 5: return "medic";
                    case 6: return "heavy";
                    case 7: return "pyro";
                    case 8: return "spy";
                    case 9: return "engineer";
                }
            }
            if (text == "demo") return "demoman";
            if (text == "engi" || text == "engie") return "engineer";
            if (text == "heavyweapons" || text == "heavyweaponsguy") return "heavy";
            return text.Replace(" ", "").Replace("_", "").Replace("-", "");
        }

        private static bool TeamValueMatches(string actual, string wanted)
        {
            if (ContainsFilterValue(actual, wanted)) return true;
            string normalizedActual = NormalizeTeamValue(actual);
            string normalizedWanted = NormalizeTeamValue(wanted);
            return normalizedActual.Length > 0 && normalizedActual == normalizedWanted;
        }

        private static string NormalizeTeamValue(string value)
        {
            string text = (value ?? "").Trim().ToLowerInvariant();
            if (text == "2" || text == "red") return "red";
            if (text == "3" || text == "blu" || text == "blue") return "blu";
            if (text == "1" || text == "spectator" || text == "spec") return "spectator";
            return text;
        }

        private static int FirstFilterSeparator(string token)
        {
            int colon = token.IndexOf(':');
            int equals = token.IndexOf('=');
            if (colon < 0) return equals;
            if (equals < 0) return colon;
            return Math.Min(colon, equals);
        }

        private static string CanonicalFilterField(string field)
        {
            string value = (field ?? "").Trim().ToLowerInvariant();
            if (value == "maps") value = "map";
            else if (value == "classes") value = "class";
            else if (value == "teams") value = "team";
            else if (value == "demos") value = "demo";
            else if (value == "weapons") value = "weapon";
            else if (value == "players") value = "player";
            else if (value == "tags") value = "tag";
            return IsFilterField(value) ? value : "";
        }

        private static bool IsFilterField(string field)
        {
            return String.Equals(field, "map", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "maps", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "class", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "classes", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "team", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "teams", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "demo", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "demos", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "weapon", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "weapons", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "player", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "players", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "tag", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "tags", StringComparison.OrdinalIgnoreCase) ||
                String.Equals(field, "text", StringComparison.OrdinalIgnoreCase);
        }

        private static bool ContainsFilterValue(string source, string value)
        {
            return !String.IsNullOrEmpty(source) && !String.IsNullOrEmpty(value) &&
                source.IndexOf(value, StringComparison.OrdinalIgnoreCase) >= 0;
        }

        private static List<string> SplitFilterTokens(string filter)
        {
            List<string> tokens = new List<string>();
            StringBuilder current = new StringBuilder();
            bool inQuotes = false;
            for (int index = 0; index < (filter ?? "").Length; index++)
            {
                char character = filter[index];
                if (character == '\"')
                {
                    inQuotes = !inQuotes;
                    current.Append(character);
                }
                else if ((Char.IsWhiteSpace(character) || character == ',') && !inQuotes)
                {
                    if (current.Length > 0)
                    {
                        tokens.Add(current.ToString());
                        current.Length = 0;
                    }
                }
                else
                {
                    current.Append(character);
                }
            }
            if (current.Length > 0) tokens.Add(current.ToString());
            return tokens;
        }

        private static string UnquoteFilterValue(string value)
        {
            if (String.IsNullOrEmpty(value)) return "";
            string text = value.Trim();
            if (text.Length >= 2 && text[0] == '\"' && text[text.Length - 1] == '\"')
                return text.Substring(1, text.Length - 2);
            return text.Replace("\"", "");
        }

        private sealed class FilterTerm
        {
            public readonly string Field;
            public readonly string Value;

            public FilterTerm(string field, string value)
            {
                Field = field;
                Value = value;
            }
        }

        private void FitCandidateColumnsToContent()
        {
            foreach (DataGridViewColumn column in grid.Columns)
            {
                if (!column.Visible) continue;
                int contentWidth = column.GetPreferredWidth(DataGridViewAutoSizeColumnMode.AllCells, true) + 8;
                int minimumWidth = Math.Max(column.Width, contentWidth);
                column.MinimumWidth = minimumWidth;
                column.Width = minimumWidth;
            }
        }

        private void UpdateRecordingEstimate()
        {
            List<IDictionary> selected = new List<IDictionary>();
            foreach (DataGridViewRow row in grid.SelectedRows)
            {
                IDictionary candidate = row.Tag as IDictionary;
                if (candidate != null) selected.Add(candidate);
            }
            if (selected.Count == 0)
            {
                recordingEstimate.Text = "Recording size estimate: select one or more candidates.";
                return;
            }
            int captureFps = Convert.ToInt32(recordingFps.SelectedItem);
            HlaeRecordingOutput output = HlaeRecordingOutputs.FromDisplayName(Convert.ToString(recordingOutput.SelectedItem));
            RecordingSizeEstimate estimate = HlaeRecordingSizeEstimator.Estimate(selected, leadInSeconds.Value, outroSeconds.Value, captureFps, output, (int)jpgQuality.Value, HlaeBatchRecorder.SavedRecordingResolution());
            recordingEstimate.Text = "Recording size estimate: " + estimate.Summary(output, (int)jpgQuality.Value);
        }

        private void LaunchSelectedCandidate()
        {
            if (grid.SelectedRows.Count == 0) return;
            IDictionary candidate = grid.SelectedRows[0].Tag as IDictionary;
            if (candidate == null) return;
            string candidateDemoPath = BatchCandidateSupport.CandidateDemoPath(candidate, demoPath);
            if (String.IsNullOrEmpty(candidateDemoPath) || !File.Exists(candidateDemoPath))
            {
                MessageBox.Show(this, "The original demo path was not found in the export manifest. Reopen the export folder or choose the demo again.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            if (String.IsNullOrEmpty(tf2Executable)) FindTf2ExecutableNearDemo(candidateDemoPath);
            if (String.IsNullOrEmpty(tf2Executable) || !File.Exists(tf2Executable))
            {
                using (OpenFileDialog dialog = new OpenFileDialog())
                {
                    dialog.Title = "Select Team Fortress 2 executable (tf_win64.exe preferred)";
                    dialog.Filter = "Team Fortress 2 (tf.exe or tf_win64.exe)|tf.exe;tf_win64.exe|Executable (*.exe)|*.exe";
                    if (dialog.ShowDialog(this) != DialogResult.OK) return;
                    tf2Executable = dialog.FileName;
                }
            }
            IList ticks = List(candidate, "point_of_kill_ticks");
            if (ticks.Count == 0) return;
            int firstTick;
            try { firstTick = Convert.ToInt32(ticks[0]); }
            catch { MessageBox.Show(this, "This candidate has no usable demo playback tick.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning); return; }
            int leadTicks = (int)Math.Round((double)leadInSeconds.Value * 66.6666667);
            int targetTick = Math.Max(0, firstTick - leadTicks);
            int attackerUserId = IntValue(candidate, "attacker_user_id");
            bool focusStvAttacker = IsStvCandidate(candidate) && attackerUserId > 0;
            try
            {
                string gameDirectory = GetTfGameDirectory();
                if (IsTf2AlreadyRunning())
                {
                    MessageBox.Show(this, "TF2 is already running. Close it before opening another candidate so the new demo and tick command are not ignored.", Text, MessageBoxButtons.OK, MessageBoxIcon.Information);
                    return;
                }
                DeleteStalePlaybackVdms(gameDirectory);
                string stagedDemo = StageDemoForPlayback(gameDirectory, candidateDemoPath);
                string playbackVdm = WritePlaybackVdm(gameDirectory, stagedDemo, targetTick, focusStvAttacker ? attackerUserId : 0);
                string arguments = "-novid -console -game tf +playdemo " + Quote(stagedDemo);
                Process launchedTf2 = Process.Start(new ProcessStartInfo
                {
                    FileName = tf2Executable,
                    Arguments = arguments,
                    UseShellExecute = true,
                    WorkingDirectory = Path.GetDirectoryName(tf2Executable)
                });
                AppendLaunchNote(candidate, targetTick, firstTick, stagedDemo, playbackVdm, focusStvAttacker, attackerUserId);
                if (launchedTf2 != null) SchedulePlaybackVdmCleanup(launchedTf2, playbackVdm);
            }
            catch (Exception error)
            {
                MessageBox.Show(this, "Could not launch TF2:\r\n" + error.Message, Text, MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private string GetTfGameDirectory()
        {
            string executableDirectory = Path.GetDirectoryName(tf2Executable);
            string gameDirectory = Path.Combine(executableDirectory, "tf");
            if (Directory.Exists(gameDirectory)) return gameDirectory;
            if (String.Equals(Path.GetFileName(executableDirectory), "tf", StringComparison.OrdinalIgnoreCase)) return executableDirectory;
            throw new DirectoryNotFoundException("Could not find TF2's tf game folder next to the selected executable. Select tf.exe from your Team Fortress 2 installation folder.");
        }

        private bool IsTf2AlreadyRunning()
        {
            string[] processNames = new string[] { Path.GetFileNameWithoutExtension(tf2Executable), "tf", "tf_win64" };
            foreach (string processName in processNames)
            {
                Process[] processes = Process.GetProcessesByName(processName);
                foreach (Process process in processes)
                {
                    try
                    {
                        if (!process.HasExited) return true;
                    }
                    catch { }
                    finally { process.Dispose(); }
                }
            }
            return false;
        }

        private string StageDemoForPlayback(string gameDirectory, string sourceDemoPath)
        {
            string demoDirectory = Path.Combine(gameDirectory, "demos", "tf2fragdemohelper");
            Directory.CreateDirectory(demoDirectory);
            string sourceName = Path.GetFileNameWithoutExtension(sourceDemoPath);
            StringBuilder safeName = new StringBuilder();
            foreach (char character in sourceName)
            {
                if (Char.IsLetterOrDigit(character) || character == '_' || character == '-') safeName.Append(character);
                else safeName.Append('_');
            }
            if (safeName.Length == 0) safeName.Append("candidate_demo");
            string stagedFileName = PlaybackTempPrefix + safeName.ToString() + "_" + new FileInfo(sourceDemoPath).Length + ".dem";
            string stagedPath = Path.Combine(demoDirectory, stagedFileName);
            if (!File.Exists(stagedPath) || new FileInfo(stagedPath).Length != new FileInfo(sourceDemoPath).Length)
                File.Copy(sourceDemoPath, stagedPath, true);
            return "demos/tf2fragdemohelper/" + stagedFileName;
        }

        private void SelectAllVisibleCandidates()
        {
            grid.ClearSelection();
            foreach (DataGridViewRow row in grid.Rows) row.Selected = true;
            summary.Text = grid.SelectedRows.Count + " candidate(s) selected for batch recording.";
            UpdateCandidateActionAvailability();
        }

        private void ClearCandidateSelection()
        {
            grid.ClearSelection();
            grid.CurrentCell = null;
            UpdateCandidateActionAvailability();
        }

        private void RememberSelectedRowClick(object sender, DataGridViewCellMouseEventArgs e)
        {
            clickedSelectedRow = -1;
            if (e.RowIndex < 0 || e.Button != MouseButtons.Left || ModifierKeys != Keys.None) return;
            if (grid.Rows[e.RowIndex].Selected) clickedSelectedRow = e.RowIndex;
        }

        private void ToggleClickedSelectedRow(object sender, DataGridViewCellEventArgs e)
        {
            if (e.RowIndex < 0 || e.RowIndex != clickedSelectedRow) return;
            clickedSelectedRow = -1;
            grid.Rows[e.RowIndex].Selected = false;
            if (grid.SelectedRows.Count == 0) grid.CurrentCell = null;
            UpdateCandidateActionAvailability();
        }

        private void UpdateCandidateActionAvailability()
        {
            int selectedCount = grid.SelectedRows.Count;
            bool hasSelection = selectedCount > 0;
            recordButton.Enabled = hasSelection;
            inlinePreviewButton.Visible = selectedCount == 1;
            UpdateRecordingEstimate();
        }

        private void RecordSelectedCandidates()
        {
            if (grid.SelectedRows.Count == 0)
            {
                MessageBox.Show(this, "Select one or more candidate rows first.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            List<IDictionary> selected = new List<IDictionary>();
            foreach (DataGridViewRow row in grid.SelectedRows)
            {
                IDictionary candidate = row.Tag as IDictionary;
                if (candidate != null) selected.Add(candidate);
            }
            selected.Reverse();
            List<IDictionary> alreadyRecorded = new List<IDictionary>();
            foreach (IDictionary candidate in selected)
            {
                if (HlaeBatchRecorder.IsCandidateAlreadyRecorded(candidate, demoPath)) alreadyRecorded.Add(candidate);
            }
            if (alreadyRecorded.Count > 0)
            {
                DialogResult choice = MessageBox.Show(this,
                    alreadyRecorded.Count + " selected candidate(s) already have a verified recording on disk.\r\n\r\n" +
                    "Yes: skip those candidates and record only the new ones.\r\n" +
                    "No: record all selected candidates again.\r\n" +
                    "Cancel: do not start recording.",
                    "Existing recordings found", MessageBoxButtons.YesNoCancel, MessageBoxIcon.Information, MessageBoxDefaultButton.Button1);
                if (choice == DialogResult.Cancel) return;
                if (choice == DialogResult.Yes)
                {
                    selected.RemoveAll(delegate(IDictionary candidate) { return HlaeBatchRecorder.IsCandidateAlreadyRecorded(candidate, demoPath); });
                    if (selected.Count == 0)
                    {
                        MessageBox.Show(this, "All selected candidates already have a verified recording on disk.", Text, MessageBoxButtons.OK, MessageBoxIcon.Information);
                        return;
                    }
                }
            }
            try
            {
                if (String.IsNullOrEmpty(tf2Executable) && selected.Count > 0)
                    FindTf2ExecutableNearDemo(BatchCandidateSupport.CandidateDemoPath(selected[0], demoPath));
                HlaeRecordingOutput output = HlaeRecordingOutputs.FromDisplayName(Convert.ToString(recordingOutput.SelectedItem));
                int captureFps = Convert.ToInt32(recordingFps.SelectedItem);
                HlaeBatchRecorder.Launch(this, selected, demoPath, tf2Executable, leadInSeconds.Value, outroSeconds.Value, captureFps, output, (int)jpgQuality.Value);
                details.AppendText("\r\nHLAE batch prepared for " + selected.Count + " selected candidate(s) as " + HlaeRecordingOutputs.DisplayName(output) + ". The launch is offline-only (-insecure, sv_lan 1).\r\n");
                ScrollDetailsToBottom();
            }
            catch (Exception error)
            {
                MessageBox.Show(this, "Could not prepare HLAE batch recording:\r\n" + error.Message, Text, MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void UpdateOutputDescription()
        {
            HlaeRecordingOutput output = HlaeRecordingOutputs.FromDisplayName(Convert.ToString(recordingOutput.SelectedItem));
            bool jpg = output == HlaeRecordingOutput.JpgSequence;
            jpgQuality.Enabled = jpg;
            string note = HlaeRecordingOutputs.DisplayName(output) + " -> " + HlaeRecordingOutputs.ExpectedFiles(output);
            if (HlaeRecordingOutputs.RequiresFfmpeg(output)) note += " (requires HLAE FFmpeg)";
            if (jpg) note += " (100 is highest quality; 90 is the default)";
            summary.Text = note;
            UpdateRecordingEstimate();
        }

        private static string WritePlaybackVdm(string gameDirectory, string stagedDemo, int targetTick, int stvAttackerUserId)
        {
            string stagedPath = Path.Combine(gameDirectory, stagedDemo.Replace('/', Path.DirectorySeparatorChar));
            string vdmPath = Path.ChangeExtension(stagedPath, ".vdm");
            List<string> lines = new List<string>();
            lines.Add("demoactions");
            lines.Add("{");
            lines.Add("    \"1\"");
            lines.Add("    {");
            lines.Add("        factory \"SkipAhead\"");
            lines.Add("        name \"TF2 Frag Demo Helper seek\"");
            lines.Add("        starttick \"1\"");
            lines.Add("        skiptotick \"" + targetTick + "\"");
            lines.Add("    }");
            if (stvAttackerUserId > 0)
            {
                lines.Add("    \"2\"");
                lines.Add("    {");
                lines.Add("        factory \"PlayCommands\"");
                lines.Add("        name \"Focus selected STV candidate\"");
                lines.Add("        starttick \"" + (targetTick + 1) + "\"");
                lines.Add("        commands \"spec_autodirector 0; spec_player #" + stvAttackerUserId + "; spec_mode 4\"");
                lines.Add("    }");
            }
            lines.Add("}");
            File.WriteAllLines(vdmPath, lines.ToArray());
            return vdmPath;
        }

        private static void DeleteStalePlaybackVdms(string gameDirectory)
        {
            string demoDirectory = Path.Combine(gameDirectory, "demos", "tf2fragdemohelper");
            if (!Directory.Exists(demoDirectory)) return;
            foreach (string vdmPath in Directory.GetFiles(demoDirectory, PlaybackTempPrefix + "*.vdm"))
            {
                try { File.Delete(vdmPath); }
                catch { }
            }
        }

        private void SchedulePlaybackVdmCleanup(Process launchedTf2, string playbackVdm)
        {
            Task.Run(delegate
            {
                try
                {
                    launchedTf2.WaitForExit();
                    Thread.Sleep(2500);
                    while (IsTf2AlreadyRunning()) Thread.Sleep(1000);
                }
                catch { }
                finally
                {
                    launchedTf2.Dispose();
                    try { if (File.Exists(playbackVdm)) File.Delete(playbackVdm); }
                    catch { }
                }
            });
        }

        private void AppendLaunchNote(IDictionary candidate, int targetTick, int firstTick, string stagedDemo, string playbackVdm, bool focusedStvAttacker, int attackerUserId)
        {
            details.AppendText("\r\nTF2 launch requested with -novid. Demo staged as " + stagedDemo + ".\r\nTemporary VDM seek script: " + playbackVdm + ". It will be removed after TF2 closes.\r\nSkipping to demo tick " + targetTick + " (" + leadInSeconds.Value + " seconds before first event at " + firstTick + ").\r\n");
            if (focusedStvAttacker)
                details.AppendText("STV camera focus: first-person view of selected candidate attacker #" + attackerUserId + ".\r\n");
            else
                details.AppendText("Camera focus: preserved recorded POV (automatic attacker focus is used only for confirmed STV demos).\r\n");
            ScrollDetailsToBottom();
        }

        private static bool IsStvCandidate(IDictionary candidate)
        {
            string captureType = TextValue(Map(candidate, "demo_context"), "capture_type");
            return String.Equals(captureType, "stv", StringComparison.OrdinalIgnoreCase);
        }

        private void ShowSelectedCandidate(object sender, EventArgs e)
        {
            if (applyingFilter) return;
            UpdateCandidateActionAvailability();
            if (grid.SelectedRows.Count == 0)
            {
                details.Clear();
                return;
            }
            IDictionary candidate = grid.SelectedRows[0].Tag as IDictionary;
            if (candidate == null) return;
            StringBuilder text = new StringBuilder();
            text.AppendLine("Candidate " + DisplayValue(candidate, "candidate_id"));
            text.AppendLine("Score " + DisplayValue(candidate, "overall_score") + " | attacker #" + DisplayValue(candidate, "attacker_user_id") + " | " + DisplayValue(candidate, "attacker_team") + " " + DisplayValue(candidate, "attacker_class"));
            text.AppendLine("Tags: " + JoinCandidateTags(Value(candidate, "tags")));
            string bookmarkComment = TextValue(candidate, "bookmark_comment");
            if (!String.IsNullOrEmpty(bookmarkComment))
                text.AppendLine("Demo bookmark at tick " + DisplayValue(candidate, "bookmark_tick") + ": " + bookmarkComment);
            text.AppendLine();
            IList kills = List(candidate, "kills");
            text.AppendLine("Kill count: " + kills.Count);
            text.AppendLine("Exact demo playback ticks: " + JoinValues(List(candidate, "point_of_kill_ticks")));
            text.AppendLine("Server analysis ticks: " + JoinValues(List(candidate, "point_of_kill_server_ticks")));
            text.AppendLine("Clip window (includes lead-in/out): " + ClipTick(candidate, "clip_start_tick", "start_tick") + " to " + ClipTick(candidate, "clip_end_tick", "end_tick") + " ticks");
            AppendScoreBreakdown(text, List(candidate, "score_breakdown"));
            AppendObjectiveEvidence(text, List(candidate, "objective_followups"));
            AppendBuildingEvidence(text, List(candidate, "building_destructions"));
            AppendDemoContext(text, Map(candidate, "demo_context"));
            AppendRoundState(text, Map(candidate, "round_state"));
            text.AppendLine();
            text.AppendLine("Kills");
            for (int i = 0; i < kills.Count; i++)
            {
                IDictionary kill = kills[i] as IDictionary;
                if (kill == null) continue;
                text.AppendLine(
                    "  " + (i + 1) + ". demo tick " + DisplayValue(kill, "tick") +
                    " (server tick " + EventTick(kill) + ", packet " + DisplayValue(kill, "packet_sequence") + ", event " + DisplayValue(kill, "event_index_in_packet") + ")" +
                    " | #" + DisplayValue(kill, "attacker_user_id") + " " + DisplayValue(kill, "attacker_team") + " " + DisplayValue(kill, "attacker_class") +
                    " -> #" + DisplayValue(kill, "victim_user_id") + " " + DisplayValue(kill, "victim_team") + " " + DisplayValue(kill, "victim_class") +
                    " | " + DisplayValue(kill, "weapon") +
                    " | assist by #" + DisplayValue(kill, "assister_user_id") +
                    " | streak " + DisplayValue(kill, "kill_streak_total") +
                    " | crit " + DisplayValue(kill, "crit_type"));
            }
            AppendStateEvidence(text, kills);
            details.Text = text.ToString();
            ScrollDetailsToBottom();
            details.SelectionLength = 0;
        }

        private void ScrollDetailsToBottom()
        {
            if (details.IsDisposed) return;
            details.SelectionStart = details.TextLength;
            details.SelectionLength = 0;
            details.ScrollToCaret();
            if (!details.IsHandleCreated || detailsScrollPending) return;
            detailsScrollPending = true;
            details.BeginInvoke(new MethodInvoker(delegate
            {
                detailsScrollPending = false;
                if (details.IsDisposed) return;
                details.SelectionStart = details.TextLength;
                details.SelectionLength = 0;
                details.ScrollToCaret();
            }));
        }

        private static void AppendBuildingEvidence(StringBuilder text, IList buildings)
        {
            text.AppendLine("Building events linked to this sequence");
            if (buildings.Count == 0)
            {
                text.AppendLine("  None (building destruction alone is not treated as a kill).");
                return;
            }
            foreach (object item in buildings)
            {
                IDictionary building = item as IDictionary;
                if (building == null) continue;
                text.AppendLine("  tick " + DisplayValue(building, "event_tick") + " | " + DisplayValue(building, "object_type") + " | attacker #" + DisplayValue(building, "attacker_user_id"));
            }
        }

        private static void AppendObjectiveEvidence(StringBuilder text, IList objectives)
        {
            text.AppendLine("Cap-secure/objective follow-up evidence");
            if (objectives.Count == 0)
            {
                text.AppendLine("  None within eight seconds after the final kill.");
                return;
            }
            foreach (object item in objectives)
            {
                IDictionary objective = item as IDictionary;
                if (objective == null) continue;
                string kind = DisplayValue(objective, "kind");
                string detail = "";
                if (String.Equals(kind, "point_capture", StringComparison.Ordinal))
                {
                    detail = " | point " + DisplayValue(objective, "point") + " " + DisplayValue(objective, "point_name");
                }
                else if (String.Equals(kind, "payload_progress", StringComparison.Ordinal))
                {
                    detail = " | pusher #" + DisplayValue(objective, "pusher_user_id") + " | distance " + DisplayValue(objective, "distance");
                }
                else if (String.Equals(kind, "capture_denial", StringComparison.Ordinal))
                {
                    detail = " | blocker #" + DisplayValue(objective, "blocker_user_id") + " | point " + DisplayValue(objective, "point") + " " + DisplayValue(objective, "point_name");
                }
                text.AppendLine("  tick " + DisplayValue(objective, "event_tick") + " | " + kind + " | team " + DisplayValue(objective, "team") + detail);
            }
        }

        private static void AppendStateEvidence(StringBuilder text, IList kills)
        {
            text.AppendLine();
            text.AppendLine("Packet-state evidence");
            bool found = false;
            foreach (object item in kills)
            {
                IDictionary kill = item as IDictionary;
                if (kill == null) continue;
                IDictionary state = Map(kill, "state_evidence");
                if (state == null || !BooleanValue(state, "state_available")) continue;
                found = true;
                text.AppendLine(
                    "  kill tick " + EventTick(kill) +
                    " | airborne " + DisplayValue(state, "victim_airborne") +
                    " | confirmed airshot " + DisplayValue(state, "confirmed_airshot") +
                    " | Uber drop " + DisplayValue(state, "confirmed_uber_drop") +
                    " | alive " + DisplayValue(state, "friendly_alive_before") + "v" + DisplayValue(state, "enemy_alive_before"));
                if (BooleanValue(state, "attacker_shield_charging") || Value(state, "attacker_recent_shield_charge_tick") != null)
                {
                    text.AppendLine(
                        "    Demoknight charge | active " + DisplayValue(state, "attacker_shield_charging") +
                        " | recent charge tick " + DisplayValue(state, "attacker_recent_shield_charge_tick") +
                        " | seconds since charge " + DisplayValue(state, "attacker_seconds_since_shield_charge"));
                }
                if (BooleanValue(state, "confirmed_kritzkrieg_boost"))
                {
                    text.AppendLine("    Kritzkrieg boost | active " + DisplayValue(state, "attacker_kritz_boosted") +
                        " | deployments " + JoinValues(Value(state, "kritzkrieg_deployments")));
                }
                if (BooleanValue(state, "attacker_blast_jumping"))
                {
                    text.AppendLine("    Market Garden evidence | attacker was blast jumping at the kill tick");
                }
                if (BooleanValue(state, "confirmed_double_donk"))
                {
                    text.AppendLine("    Double Donk | direct impact then Mini-Crit explosion " + JoinValues(Value(state, "double_donk_events")));
                }
                int recentFriendlyDeaths = IntValue(state, "recent_friendly_death_count");
                if (recentFriendlyDeaths > 0 || BooleanValue(state, "enemy_uber_advantage_before"))
                {
                    text.AppendLine(
                        "    sack context | " + recentFriendlyDeaths + " recent friendly deaths in " + DisplayValue(state, "sack_recovery_window_seconds") + "s" +
                        " | player deficit " + DisplayValue(state, "player_disadvantage_before") +
                        " | enemy Uber advantage " + DisplayValue(state, "enemy_uber_advantage_before") +
                        " | Medic charge " + DisplayValue(state, "friendly_medic_charge_before") + "% vs " + DisplayValue(state, "enemy_medic_charge_before") + "%");
                }
                IList forceFollowups = List(state, "enemy_medic_force_followups");
                foreach (object forceItem in forceFollowups)
                {
                    IDictionary force = forceItem as IDictionary;
                    if (force == null) continue;
                    text.AppendLine(
                        "    Medic force | enemy Medic #" + DisplayValue(force, "medic_user_id") +
                        " (" + DisplayValue(force, "medic_team") + " vs " + DisplayValue(force, "forced_by_team") + ")" +
                        " deployed at tick " + DisplayValue(force, "event_tick") +
                        " | reference player #" + DisplayValue(force, "reference_player_user_id") +
                        " | target #" + DisplayValue(force, "target_user_id") +
                        " | pressure ticks " + JoinValues(Value(force, "pressure_event_ticks")) +
                        " | charge before sequence " + DisplayValue(force, "charge_before_sequence") + "%");
                }
                string respawnSeconds = TextValue(state, "victim_respawn_seconds");
                if (!String.IsNullOrEmpty(respawnSeconds))
                    text.AppendLine("    victim respawn observed in " + respawnSeconds + "s at tick " + DisplayValue(state, "victim_next_respawn_tick"));
                IDictionary projectile = Map(state, "projectile");
                if (projectile != null)
                {
                    text.AppendLine(
                        "    projectile #" + DisplayValue(projectile, "entity_id") +
                        " " + DisplayValue(projectile, "projectile_type") +
                        " | distance " + DisplayValue(projectile, "distance_to_victim") +
                        " | " + DisplayValue(projectile, "impact_proximity") +
                        " | flight state " + DisplayValue(projectile, "flight_state") +
                        " | flight " + DisplayValue(projectile, "flight_seconds") + "s" +
                        " | path " + DisplayValue(projectile, "tracked_path_distance") +
                        " | launcher " + DisplayValue(projectile, "launcher_handle"));
                }
            }
            if (!found) text.AppendLine("  No reconstructed player/projectile state was available for these kills.");
        }

        private static void AppendScoreBreakdown(StringBuilder text, IList breakdown)
        {
            text.AppendLine("Score breakdown");
            if (breakdown.Count == 0)
            {
                text.AppendLine("  Not available in this candidate file.");
                return;
            }
            foreach (object item in breakdown)
            {
                IDictionary contribution = item as IDictionary;
                if (contribution == null) continue;
                string points = DisplayValue(contribution, "points");
                if (!points.StartsWith("-", StringComparison.Ordinal)) points = "+" + points;
                string eventTick = TextValue(contribution, "event_tick");
                string count = TextValue(contribution, "count");
                text.AppendLine("  " + points + "  " + CandidateReasonName(DisplayValue(contribution, "reason")) +
                    (String.IsNullOrEmpty(count) ? "" : " (count " + count + ")") +
                    (String.IsNullOrEmpty(eventTick) ? "" : " at tick " + eventTick));
            }
        }

        private static void AppendRoundState(StringBuilder text, IDictionary state)
        {
            if (state == null) return;
            text.AppendLine("Round evidence");
            text.AppendLine("  playable start: " + DisplayValue(state, "start_tick") + " (" + DisplayValue(state, "start_event") + ")");
            IDictionary trigger = Map(state, "activation_trigger");
            text.AppendLine("  activation trigger: " + DisplayValue(trigger, "event") + " at " + DisplayValue(trigger, "tick"));
            text.AppendLine("  round-active tick: " + DisplayValue(state, "round_active_tick") + " | setup-finished tick: " + DisplayValue(state, "setup_finished_tick"));
            text.AppendLine("  round end: " + DisplayValue(state, "end_tick") + " (" + DisplayValue(state, "end_event") + ")");
            IDictionary ready = Map(state, "ready_up");
            text.AppendLine("  ready-up: RED " + DisplayValue(ready, "red_ready_tick") + ", BLU " + DisplayValue(ready, "blu_ready_tick") + ", both ready " + DisplayValue(ready, "both_teams_ready"));
        }

        private static void AppendDemoContext(StringBuilder text, IDictionary context)
        {
            if (context == null) return;
            text.AppendLine("Demo scope: " + DisplayValue(context, "capture_type") + " (" + DisplayValue(context, "capture_confidence") + ") | " + DisplayValue(context, "analysis_scope"));
            text.AppendLine("Mode: " + DisplayValue(context, "mode_label") + " (" + DisplayValue(context, "mode_confidence") + ")");
            IList modeEvidence = List(context, "mode_evidence");
            foreach (object item in modeEvidence) text.AppendLine("  mode evidence: " + Convert.ToString(item));
            text.AppendLine("  POV recorder: " + DisplayValue(context, "header_nick") + " | user ID " + DisplayValue(context, "pov_player_user_id"));
            text.AppendLine("  scope reason: " + DisplayValue(context, "scope_reason"));
        }

        private static object Value(IDictionary values, string key)
        {
            return values != null && values.Contains(key) ? values[key] : null;
        }

        private static IDictionary Map(IDictionary values, string key)
        {
            return Value(values, key) as IDictionary;
        }

        private static IList List(IDictionary values, string key)
        {
            IList result = Value(values, key) as IList;
            return result ?? new ArrayList();
        }

        private static string TextValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            return value == null ? "" : Convert.ToString(value);
        }

        private static bool BooleanValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            try { return value != null && Convert.ToBoolean(value); }
            catch (Exception) { return false; }
        }

        private static int IntValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            try { return value == null ? 0 : Convert.ToInt32(value); }
            catch (Exception) { return 0; }
        }

        private static string DisplayValue(IDictionary values, string key)
        {
            string value = TextValue(values, key);
            return String.IsNullOrEmpty(value) ? "Unknown" : value;
        }

        private static string EventTick(IDictionary values)
        {
            string eventTick = TextValue(values, "event_tick");
            return String.IsNullOrEmpty(eventTick) ? DisplayValue(values, "tick") : eventTick;
        }

        private static string ClipTick(IDictionary values, string preferredKey, string legacyKey)
        {
            string value = TextValue(values, preferredKey);
            return String.IsNullOrEmpty(value) ? DisplayValue(values, legacyKey) : value;
        }

        private static string JoinValues(object values)
        {
            IList list = values as IList;
            if (list == null || list.Count == 0) return "None";
            List<string> text = new List<string>();
            foreach (object value in list) text.Add(Convert.ToString(value));
            return String.Join(", ", text.ToArray());
        }

        private static string JoinCandidateTags(object values)
        {
            IList list = values as IList;
            if (list == null || list.Count == 0) return "None";
            List<string> text = new List<string>();
            foreach (object value in list) text.Add(CandidateTagName(Convert.ToString(value)));
            return String.Join(", ", text.ToArray());
        }

        private static string CandidateTagName(string tag)
        {
            if (String.Equals(tag, "kills_to_secure_cap", StringComparison.Ordinal) ||
                String.Equals(tag, "cap_secure_kills", StringComparison.Ordinal) ||
                String.Equals(tag, "objective_capture_followup", StringComparison.Ordinal))
                return "kills_to_secure_cap";
            return tag;
        }

        private static string CandidateReasonName(string reason)
        {
            if (String.Equals(reason, "kills_to_secure_cap", StringComparison.Ordinal) ||
                String.Equals(reason, "cap_secure_kills", StringComparison.Ordinal) ||
                String.Equals(reason, "kill_sequence_led_to_point_capture", StringComparison.Ordinal))
                return "kills_to_secure_cap (capture followed this sequence)";
            return reason;
        }

        private sealed class CandidateRecord
        {
            public readonly int Rank;
            public readonly IDictionary Candidate;
            public readonly decimal Score;
            public readonly string SearchText;

            public CandidateRecord(int rank, IDictionary candidate, string sourceLine)
            {
                Rank = rank;
                Candidate = candidate;
                Score = DecimalValue(candidate, "overall_score");
                SearchText = sourceLine ?? "";
            }
        }

        private static decimal DecimalValue(IDictionary values, string key)
        {
            object value = Value(values, key);
            try { return Convert.ToDecimal(value); }
            catch (Exception) { return 0; }
        }
    }
}
