using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
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
            Application.Run(new MainForm());
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
        private Process activeProcess;
        private bool busy;
        private string lastExport;
        private readonly List<string> selectedDemos = new List<string>();
        private string demoSelectionDisplay = "";

        public MainForm()
        {
            root = AppDomain.CurrentDomain.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            Text = "TF2 STV Parser";
            StartPosition = FormStartPosition.CenterScreen;
            // The integrated candidate browser has a wide grid and a
            // dedicated Back button. Keep the shared window large enough for
            // those controls at normal Windows scaling.
            MinimumSize = new Size(1200, 680);
            Size = new Size(1280, 780);
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

            Label note = Label("Select one or more demos. Batch parsing runs them in order and creates one combined candidate list. Weapon slots use TF2's current items_game.txt when available.");
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
            busy = true;
            parseButton.Enabled = false;
            cancelButton.Enabled = true;
            openButton.Enabled = false;
            candidatesButton.Enabled = false;
            log.Clear();
            progress.Style = ProgressBarStyle.Marquee;
            status.Text = "Parsing " + demos.Count + " demo" + (demos.Count == 1 ? "" : "s") + "...";
            status.ForeColor = Color.Gainsboro;
            Append("Inputs: " + demos.Count + " demo(s)\r\nExport: " + lastExport + "\r\n\r\n");
            try
            {
                List<BatchExportEntry> batchExports = new List<BatchExportEntry>();
                for (int index = 0; index < demos.Count; index++)
                {
                    string demo = demos[index];
                    string exportDirectory = batch
                        ? Path.Combine(lastExport, (index + 1).ToString("D3") + "_" + SafeFileName(Path.GetFileNameWithoutExtension(demo)) + "_export")
                        : lastExport;
                    Directory.CreateDirectory(exportDirectory);
                    status.Text = "Parsing demo " + (index + 1) + " of " + demos.Count + ": " + Path.GetFileName(demo);
                    Append("\r\n[" + (index + 1) + "/" + demos.Count + "] Input: " + demo + "\r\nExport: " + exportDirectory + "\r\n");
                    await RunWorker(parser, Quote(demo) + " " + Quote(exportDirectory));
                    status.Text = "Ranking candidates for demo " + (index + 1) + " of " + demos.Count + "...";
                    Append("Running frag analysis...\r\n");
                    await RunFragAnalysis(exportDirectory);
                    batchExports.Add(new BatchExportEntry(index + 1, demo, exportDirectory));
                }
                if (batch)
                {
                    int count = BatchCandidateSupport.WriteCombinedExport(lastExport, batchExports);
                    Append("\r\nCombined " + count + " candidates from " + demos.Count + " demos into " + Path.Combine(lastExport, "frag_candidates.ndjson") + ".\r\n");
                }
                progress.Style = ProgressBarStyle.Continuous;
                progress.Value = 100;
                status.Text = batch ? "Batch export and combined candidate analysis complete." : "Export and frag analysis complete.";
                status.ForeColor = Color.LightGreen;
                openButton.Enabled = true;
                candidatesButton.Enabled = true;
                Append("\r\nSUCCESS: Export and frag analysis complete. Use View candidates to inspect and batch-record ranked clips.\r\n");
            }
            catch (Exception ex)
            {
                progress.Style = ProgressBarStyle.Continuous;
                progress.Value = 0;
                status.Text = "Failed: " + ex.Message;
                status.ForeColor = Color.OrangeRed;
                openButton.Enabled = Directory.Exists(outputBox.Text);
                Append("\r\nERROR: " + ex.Message + "\r\n");
            }
            finally
            {
                busy = false;
                parseButton.Enabled = true;
                cancelButton.Enabled = false;
            }
        }

        private Task RunWorker(string fileName, string arguments)
        {
            return Task.Run(delegate
            {
                ProcessStartInfo info = new ProcessStartInfo();
                info.FileName = fileName;
                info.Arguments = arguments;
                info.WorkingDirectory = root;
                info.UseShellExecute = false;
                info.CreateNoWindow = true;
                info.RedirectStandardOutput = true;
                info.RedirectStandardError = true;
                Process process = new Process();
                process.StartInfo = info;
                process.OutputDataReceived += delegate(object sender, DataReceivedEventArgs e) { if (e.Data != null) Append(e.Data + "\r\n"); };
                process.ErrorDataReceived += delegate(object sender, DataReceivedEventArgs e) { if (e.Data != null) Append(e.Data + "\r\n"); };
                activeProcess = process;
                process.Start();
                process.BeginOutputReadLine();
                process.BeginErrorReadLine();
                process.WaitForExit();
                int code = process.ExitCode;
                activeProcess = null;
                process.Dispose();
                if (code != 0) throw new InvalidOperationException("Parser exited with code " + code + ". See the log.");
            });
        }

        private async Task RunFragAnalysis(string exportDirectory)
        {
            string script = Path.Combine(root, "analyze_frags.py");
            if (!File.Exists(script)) throw new FileNotFoundException("Frag analyzer is missing.", script);
            string analyzerArguments = Quote(script) + " --debug ";
            if (!String.IsNullOrWhiteSpace(schemaBox.Text))
                analyzerArguments += "--item-schema " + Quote(schemaBox.Text.Trim()) + " ";
            analyzerArguments += Quote(exportDirectory);
            Exception pythonFailure = null;
            try
            {
                await RunWorker("python.exe", analyzerArguments);
            }
            catch (Exception ex)
            {
                pythonFailure = ex;
            }
            if (pythonFailure != null)
                await RunWorker("py.exe", "-3 " + analyzerArguments);
        }

        private void Cancel(object sender, EventArgs e)
        {
            Process process = activeProcess;
            if (process == null || process.HasExited) return;
            cancelButton.Enabled = false;
            status.Text = "Cancelling parser or frag analysis...";
            Task.Run(delegate { try { process.Kill(); } catch { } });
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
        }

        private static string Quote(string value) { return "\"" + value.Replace("\"", "\\\"") + "\""; }
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
        private readonly NumericUpDown recordingFps = new NumericUpDown();
        private readonly NumericUpDown jpgQuality = new NumericUpDown();
        private readonly ComboBox recordingOutput = new ComboBox();
        private readonly DataGridViewButtonColumn previewColumn = new DataGridViewButtonColumn();
        private readonly Button selectAllButton = GreenButton("Select all visible", 135);
        private readonly Button recordButton = GreenButton("Record selected with HLAE", 205);
        private readonly Button backButton = GreenButton("Back to parser", 165);
        private readonly List<CandidateRecord> records = new List<CandidateRecord>();
        private string demoPath;
        private string tf2Executable;
        private bool detailsScrollPending;
        private int clickedSelectedRow = -1;

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
            MinimumSize = new Size(1200, 680);
            Size = new Size(1280, 780);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;
            BuildPage();
            LoadCandidates();
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
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 152));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            Controls.Add(layout);

            TableLayoutPanel filters = new TableLayoutPanel();
            filters.Dock = DockStyle.Fill;
            filters.ColumnCount = 1;
            filters.RowCount = 4;
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            filters.RowStyles.Add(new RowStyle(SizeType.Absolute, 40));
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
            filterLabel.Text = "Filter candidates (tags, class, team, weapon, or player ID)";
            filterLabel.AutoSize = true;
            filterLabel.Margin = new Padding(3, 9, 4, 2);
            filterControls.Controls.Add(filterLabel);
            filterBox.Width = 230;
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
            playbackControls.Controls.Add(outroSeconds);
            Label fpsLabel = new Label();
            fpsLabel.Text = "Record FPS";
            fpsLabel.AutoSize = true;
            fpsLabel.Margin = new Padding(4, 9, 4, 2);
            playbackControls.Controls.Add(fpsLabel);
            recordingFps.Width = 58;
            recordingFps.Minimum = 30;
            recordingFps.Maximum = 1200;
            recordingFps.Value = 120;
            recordingFps.Increment = 30;
            recordingFps.Margin = new Padding(0, 5, 10, 2);
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
            recordingOutput.Items.Add("TGA image sequence");
            recordingOutput.Items.Add("JPG image sequence");
            recordingOutput.Items.Add("MP4 - standard");
            recordingOutput.Items.Add("MP4 - compatible");
            recordingOutput.Items.Add("MP4 - lossless");
            recordingOutput.Items.Add("AVI - raw");
            recordingOutput.SelectedIndex = 0;
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
            recordingControls.Controls.Add(jpgQuality);
            selectAllButton.Margin = new Padding(4, 3, 2, 2);
            selectAllButton.Click += delegate { SelectAllVisibleCandidates(); };
            recordingControls.Controls.Add(selectAllButton);
            recordButton.Margin = new Padding(4, 3, 2, 2);
            recordButton.Click += delegate { RecordSelectedCandidates(); };
            recordingControls.Controls.Add(recordButton);
            filters.Controls.Add(recordingControls, 0, 3);
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
            grid.SelectionMode = DataGridViewSelectionMode.FullRowSelect;
            grid.MultiSelect = true;
            grid.AutoGenerateColumns = false;
            grid.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.None;
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
            AddColumn("Exact kill-event ticks", 175);
            AddColumn("Tags", 300);
            AddPreviewColumn();
            grid.SelectionChanged += ShowSelectedCandidate;
            grid.CellMouseDown += RememberSelectedRowClick;
            grid.CellClick += ToggleClickedSelectedRow;
            grid.CellContentClick += PreviewSelectedCandidate;
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
            column.HeaderText = name;
            column.Width = width;
            column.SortMode = DataGridViewColumnSortMode.NotSortable;
            grid.Columns.Add(column);
        }

        private void AddPreviewColumn()
        {
            previewColumn.HeaderText = "";
            previewColumn.Name = "PreviewSelectedInTf2";
            previewColumn.Width = 175;
            previewColumn.Text = "Preview Selected in TF2";
            previewColumn.UseColumnTextForButtonValue = false;
            previewColumn.FlatStyle = FlatStyle.Flat;
            previewColumn.Visible = false;
            grid.Columns.Add(previewColumn);
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
            string filter = filterBox.Text.Trim().ToLowerInvariant();
            decimal requiredScore = minimumScore.Value;
            grid.Rows.Clear();
            int visible = 0;
            foreach (CandidateRecord record in records)
            {
                if (record.Score < requiredScore) continue;
                if (filter.Length > 0 && record.SearchText.IndexOf(filter, StringComparison.OrdinalIgnoreCase) < 0) continue;
                IDictionary candidate = record.Candidate;
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
                    JoinValues(killTicks),
                    JoinCandidateTags(Value(candidate, "tags")),
                    "");
                grid.Rows[row].Tag = candidate;
                visible++;
            }
            summary.Text = visible + " of " + records.Count + " ranked candidates. Select one or more rows before recording.";
            ClearCandidateSelection();
            if (grid.Rows.Count == 0)
                details.Text = records.Count == 0 ? "No candidates were produced for this demo." : "No candidates match the current filter.";
            else
                details.Clear();
            UpdateCandidateActionAvailability();
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
            if (e.ColumnIndex == previewColumn.Index) return;
            if (grid.Rows[e.RowIndex].Selected) clickedSelectedRow = e.RowIndex;
        }

        private void ToggleClickedSelectedRow(object sender, DataGridViewCellEventArgs e)
        {
            if (e.RowIndex != clickedSelectedRow) return;
            clickedSelectedRow = -1;
            grid.Rows[e.RowIndex].Selected = false;
            if (grid.SelectedRows.Count == 0) grid.CurrentCell = null;
            UpdateCandidateActionAvailability();
        }

        private void PreviewSelectedCandidate(object sender, DataGridViewCellEventArgs e)
        {
            if (e.RowIndex >= 0 && e.ColumnIndex == previewColumn.Index && grid.SelectedRows.Count == 1)
                LaunchSelectedCandidate();
        }

        private void UpdateCandidateActionAvailability()
        {
            int selectedCount = grid.SelectedRows.Count;
            bool hasSelection = selectedCount > 0;
            recordButton.Enabled = hasSelection;
            previewColumn.Visible = selectedCount == 1;
            foreach (DataGridViewRow row in grid.Rows)
                row.Cells[previewColumn.Index].Value = selectedCount == 1 && row.Selected ? "Preview Selected in TF2" : "";
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
            try
            {
                if (String.IsNullOrEmpty(tf2Executable) && selected.Count > 0)
                    FindTf2ExecutableNearDemo(BatchCandidateSupport.CandidateDemoPath(selected[0], demoPath));
                HlaeRecordingOutput output = HlaeRecordingOutputs.FromDisplayName(Convert.ToString(recordingOutput.SelectedItem));
                HlaeBatchRecorder.Launch(this, selected, demoPath, tf2Executable, leadInSeconds.Value, outroSeconds.Value, (int)recordingFps.Value, output, (int)jpgQuality.Value);
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
