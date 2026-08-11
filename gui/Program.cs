using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Text;
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
        private readonly TextBox log = new TextBox();
        private readonly Button parseButton = GreenButton("Parse STV demo", 150);
        private readonly Button cancelButton = GreenButton("Cancel", 90);
        private readonly Button openButton = GreenButton("Open export folder", 150);
        private readonly Button candidatesButton = GreenButton("View candidates", 130);
        private readonly ProgressBar progress = new ProgressBar();
        private readonly Label status = new Label();
        private Process activeProcess;
        private bool busy;
        private string lastExport;

        public MainForm()
        {
            root = AppDomain.CurrentDomain.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            Text = "TF2 STV Parser";
            StartPosition = FormStartPosition.CenterScreen;
            MinimumSize = new Size(900, 560);
            Size = new Size(1050, 700);
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
            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.Padding = new Padding(20);
            layout.ColumnCount = 3;
            layout.RowCount = 7;
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 125));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 155));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 50));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 48));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 28));
            Controls.Add(layout);

            layout.Controls.Add(Label("STV demo"), 0, 0);
            demoBox.Dock = DockStyle.Fill;
            demoBox.TextChanged += delegate { SuggestOutput(); };
            layout.Controls.Add(demoBox, 1, 0);
            Button browseDemo = GreenButton("Browse demo", 145);
            browseDemo.Click += BrowseDemo;
            layout.Controls.Add(browseDemo, 2, 0);

            layout.Controls.Add(Label("Export location"), 0, 1);
            outputBox.Dock = DockStyle.Fill;
            outputBox.TextChanged += delegate { openButton.Enabled = Directory.Exists(outputBox.Text); };
            layout.Controls.Add(outputBox, 1, 1);
            Button browseOutput = GreenButton("Select location", 145);
            browseOutput.Click += BrowseOutput;
            layout.Controls.Add(browseOutput, 2, 1);

            Label note = Label("Exports decoded data and ranks live-round frag candidates. The first analysis pass finds multi-kills, projectile kills, Medic picks, killstreaks, and random-crit flags. Airshot confirmation follows in the packet-state pass.");
            note.MaximumSize = new Size(850, 0);
            note.ForeColor = Color.Silver;
            layout.SetColumnSpan(note, 3);
            layout.Controls.Add(note, 0, 2);

            Label logTitle = Label("Parser log");
            layout.SetColumnSpan(logTitle, 3);
            layout.Controls.Add(logTitle, 0, 3);
            log.Dock = DockStyle.Fill;
            log.Multiline = true;
            log.ReadOnly = true;
            log.ScrollBars = ScrollBars.Both;
            log.WordWrap = false;
            log.BackColor = Color.FromArgb(17, 18, 20);
            log.ForeColor = Color.FromArgb(218, 224, 230);
            log.Font = new Font("Consolas", 9F);
            layout.SetColumnSpan(log, 3);
            layout.Controls.Add(log, 0, 4);

            FlowLayoutPanel actions = new FlowLayoutPanel();
            actions.Dock = DockStyle.Fill;
            parseButton.Click += async delegate { await ParseDemo(); };
            cancelButton.Enabled = false;
            cancelButton.Click += Cancel;
            openButton.Enabled = false;
            openButton.Click += OpenExport;
            candidatesButton.Enabled = false;
            candidatesButton.Click += OpenCandidates;
            actions.Controls.Add(parseButton);
            actions.Controls.Add(cancelButton);
            actions.Controls.Add(openButton);
            actions.Controls.Add(candidatesButton);
            layout.SetColumnSpan(actions, 3);
            layout.Controls.Add(actions, 0, 5);

            status.Dock = DockStyle.Fill;
            layout.SetColumnSpan(status, 3);
            layout.Controls.Add(status, 0, 6);
            progress.Dock = DockStyle.Bottom;
            progress.Height = 14;
            progress.Style = ProgressBarStyle.Continuous;
            Controls.Add(progress);
        }

        private async Task ParseDemo()
        {
            if (busy) return;
            string demo = demoBox.Text.Trim();
            if (!File.Exists(demo) || !demo.EndsWith(".dem", StringComparison.OrdinalIgnoreCase))
            {
                MessageBox.Show(this, "Choose an existing .dem file.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
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

            lastExport = Path.Combine(outputBox.Text.Trim(), Path.GetFileNameWithoutExtension(demo) + "_export_" + DateTime.Now.ToString("yyyyMMdd_HHmmss"));
            Directory.CreateDirectory(lastExport);
            busy = true;
            parseButton.Enabled = false;
            cancelButton.Enabled = true;
            openButton.Enabled = false;
            candidatesButton.Enabled = false;
            log.Clear();
            progress.Style = ProgressBarStyle.Marquee;
            status.Text = "Parsing demo...";
            status.ForeColor = Color.Gainsboro;
            Append("Input: " + demo + "\r\nExport: " + lastExport + "\r\n\r\n");
            try
            {
                await RunWorker(parser, Quote(demo) + " " + Quote(lastExport));
                status.Text = "Ranking live-round frag candidates...";
                Append("\r\nRunning frag analysis...\r\n");
                await RunFragAnalysis(lastExport);
                progress.Style = ProgressBarStyle.Continuous;
                progress.Value = 100;
                status.Text = "Export and frag analysis complete.";
                status.ForeColor = Color.LightGreen;
                openButton.Enabled = true;
                candidatesButton.Enabled = true;
                Append("\r\nSUCCESS: Export and frag analysis complete. Use View candidates to inspect ranked clips.\r\n");
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
            Exception pythonFailure = null;
            try
            {
                await RunWorker("python.exe", Quote(script) + " --debug " + Quote(exportDirectory));
            }
            catch (Exception ex)
            {
                pythonFailure = ex;
            }
            if (pythonFailure != null)
                await RunWorker("py.exe", "-3 " + Quote(script) + " --debug " + Quote(exportDirectory));
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
                if (dialog.ShowDialog(this) == DialogResult.OK) demoBox.Text = dialog.FileName;
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

        private void SuggestOutput()
        {
            if (File.Exists(demoBox.Text)) outputBox.Text = Path.GetDirectoryName(demoBox.Text);
        }

        private void OpenExport(object sender, EventArgs e)
        {
            string target = Directory.Exists(lastExport) ? lastExport : outputBox.Text;
            if (Directory.Exists(target)) Process.Start("explorer.exe", Quote(target));
        }

        private void OpenCandidates(object sender, EventArgs e)
        {
            if (String.IsNullOrEmpty(lastExport)) return;
            string path = Path.Combine(lastExport, "frag_candidates.ndjson");
            if (!File.Exists(path))
            {
                MessageBox.Show(this, "frag_candidates.ndjson was not found in the latest export.", Text, MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            CandidateViewerForm viewer = new CandidateViewerForm(path);
            viewer.Show(this);
        }

        private void OnDragEnter(object sender, DragEventArgs e)
        {
            if (e.Data.GetDataPresent(DataFormats.FileDrop)) e.Effect = DragDropEffects.Copy;
        }

        private void OnDragDrop(object sender, DragEventArgs e)
        {
            string[] files = e.Data.GetData(DataFormats.FileDrop) as string[];
            if (files != null && files.Length > 0 && files[0].EndsWith(".dem", StringComparison.OrdinalIgnoreCase)) demoBox.Text = files[0];
        }

        private void Append(string text)
        {
            if (log.InvokeRequired) { log.BeginInvoke(new Action<string>(Append), text); return; }
            log.AppendText(text);
        }

        private static string Quote(string value) { return "\"" + value.Replace("\"", "\\\"") + "\""; }
    }

    internal sealed class CandidateViewerForm : Form
    {
        private readonly string candidatesPath;
        private readonly DataGridView grid = new DataGridView();
        private readonly TextBox details = new TextBox();
        private readonly Label summary = new Label();
        private readonly TextBox filterBox = new TextBox();
        private readonly NumericUpDown minimumScore = new NumericUpDown();
        private readonly List<CandidateRecord> records = new List<CandidateRecord>();

        public CandidateViewerForm(string path)
        {
            candidatesPath = path;
            Text = "TF2 Frag Candidates";
            StartPosition = FormStartPosition.CenterScreen;
            MinimumSize = new Size(1050, 620);
            Size = new Size(1350, 820);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;
            BuildPage();
            LoadCandidates();
        }

        private void BuildPage()
        {
            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.Padding = new Padding(14);
            layout.ColumnCount = 1;
            layout.RowCount = 2;
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 58));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            Controls.Add(layout);

            FlowLayoutPanel filters = new FlowLayoutPanel();
            filters.Dock = DockStyle.Fill;
            filters.FlowDirection = FlowDirection.LeftToRight;
            filters.WrapContents = true;
            summary.AutoSize = true;
            summary.Margin = new Padding(3, 7, 18, 2);
            summary.ForeColor = Color.Silver;
            filters.Controls.Add(summary);
            Label filterLabel = new Label();
            filterLabel.Text = "Filter";
            filterLabel.AutoSize = true;
            filterLabel.Margin = new Padding(3, 9, 4, 2);
            filters.Controls.Add(filterLabel);
            filterBox.Width = 250;
            filterBox.Margin = new Padding(0, 5, 14, 2);
            filterBox.TextChanged += delegate { ApplyFilter(); };
            filters.Controls.Add(filterBox);
            Label minimumLabel = new Label();
            minimumLabel.Text = "Minimum score";
            minimumLabel.AutoSize = true;
            minimumLabel.Margin = new Padding(3, 9, 4, 2);
            filters.Controls.Add(minimumLabel);
            minimumScore.Width = 70;
            minimumScore.Maximum = 1000;
            minimumScore.Margin = new Padding(0, 5, 2, 2);
            minimumScore.ValueChanged += delegate { ApplyFilter(); };
            filters.Controls.Add(minimumScore);
            Label hint = new Label();
            hint.Text = "Matches tags, class, team, weapon, or player ID";
            hint.AutoSize = true;
            hint.ForeColor = Color.Gray;
            hint.Margin = new Padding(12, 9, 2, 2);
            filters.Controls.Add(hint);
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
            grid.MultiSelect = false;
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
            AddColumn("Exact kill-event ticks", 175);
            AddColumn("Tags", 430);
            grid.SelectionChanged += ShowSelectedCandidate;
            split.Panel1.Controls.Add(grid);

            details.Dock = DockStyle.Fill;
            details.Multiline = true;
            details.ReadOnly = true;
            details.ScrollBars = ScrollBars.Both;
            details.WordWrap = false;
            details.BackColor = Color.FromArgb(17, 18, 20);
            details.ForeColor = Color.FromArgb(218, 224, 230);
            details.Font = new Font("Consolas", 10F);
            split.Panel2.Controls.Add(details);
        }

        private void AddColumn(string name, int width)
        {
            DataGridViewTextBoxColumn column = new DataGridViewTextBoxColumn();
            column.HeaderText = name;
            column.Width = width;
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
            ApplyFilter();
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
                    JoinValues(killTicks),
                    JoinValues(Value(candidate, "tags")));
                grid.Rows[row].Tag = candidate;
                visible++;
            }
            summary.Text = visible + " of " + records.Count + " ranked candidates. Kill-event ticks are exact; clip boundaries include lead-in and follow-through.";
            if (grid.Rows.Count > 0) grid.Rows[0].Selected = true;
            else details.Text = records.Count == 0 ? "No candidates were produced for this demo." : "No candidates match the current filter.";
        }

        private void ShowSelectedCandidate(object sender, EventArgs e)
        {
            if (grid.SelectedRows.Count == 0) return;
            IDictionary candidate = grid.SelectedRows[0].Tag as IDictionary;
            if (candidate == null) return;
            StringBuilder text = new StringBuilder();
            text.AppendLine("Candidate " + DisplayValue(candidate, "candidate_id"));
            text.AppendLine("Score " + DisplayValue(candidate, "overall_score") + " | attacker #" + DisplayValue(candidate, "attacker_user_id") + " | " + DisplayValue(candidate, "attacker_team") + " " + DisplayValue(candidate, "attacker_class"));
            text.AppendLine("Tags: " + JoinValues(Value(candidate, "tags")));
            text.AppendLine();
            IList kills = List(candidate, "kills");
            text.AppendLine("Kill count: " + kills.Count);
            text.AppendLine("Exact player_death ticks: " + JoinValues(List(candidate, "point_of_kill_ticks")));
            text.AppendLine("Clip window (includes lead-in/out): " + ClipTick(candidate, "clip_start_tick", "start_tick") + " to " + ClipTick(candidate, "clip_end_tick", "end_tick") + " ticks");
            AppendScoreBreakdown(text, List(candidate, "score_breakdown"));
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
                    "  " + (i + 1) + ". event tick " + EventTick(kill) +
                    " (packet " + DisplayValue(kill, "packet_sequence") + ", event " + DisplayValue(kill, "event_index_in_packet") + ")" +
                    " | #" + DisplayValue(kill, "attacker_user_id") + " " + DisplayValue(kill, "attacker_team") + " " + DisplayValue(kill, "attacker_class") +
                    " -> #" + DisplayValue(kill, "victim_user_id") + " " + DisplayValue(kill, "victim_team") + " " + DisplayValue(kill, "victim_class") +
                    " | " + DisplayValue(kill, "weapon") +
                    " | assist by #" + DisplayValue(kill, "assister_user_id") +
                    " | streak " + DisplayValue(kill, "kill_streak_total") +
                    " | crit " + DisplayValue(kill, "crit_type"));
            }
            details.Text = text.ToString();
            details.SelectionStart = 0;
            details.SelectionLength = 0;
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
                text.AppendLine("  " + points + "  " + DisplayValue(contribution, "reason") +
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
