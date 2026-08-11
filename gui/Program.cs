using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Text;
using System.Threading.Tasks;
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
            actions.Controls.Add(parseButton);
            actions.Controls.Add(cancelButton);
            actions.Controls.Add(openButton);
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
                Append("\r\nSUCCESS: Export and frag analysis complete. Open frag_candidates.ndjson to see ranked clips.\r\n");
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
                await RunWorker("python.exe", Quote(script) + " " + Quote(exportDirectory));
            }
            catch (Exception ex)
            {
                pythonFailure = ex;
            }
            if (pythonFailure != null)
                await RunWorker("py.exe", "-3 " + Quote(script) + " " + Quote(exportDirectory));
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
}
