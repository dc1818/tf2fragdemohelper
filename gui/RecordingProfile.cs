using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows.Forms;
using Microsoft.Win32;

namespace Tf2StvParserGui
{
    internal sealed class HlaeRecordingSettings
    {
        public string FfmpegExecutable = "";
        public string HlaeExecutable = "";
        public string Tf2Executable = "";
        public string OutputDirectory = "";
        public string LawenaResourcesDirectory = "";
        public string Resolution = "2560x1440";
        public string DxLevel = "98 (Lawena highest)";
        public string Skybox = "Default";
        public string Hud = "Kill notices only";
        public string Viewmodels = "On";
        public int ViewmodelFov = 70;
        public bool MaximumGraphics = true;
        public bool MotionBlur = true;
        public bool DisableHitSounds = true;
        public bool DisableVoiceChat = true;
        public bool MinimalHud = true;
        public bool DisableCombatText = true;
        public bool DisableCrosshair = true;
        public bool DisableCrosshairSwitching = true;
        public bool HudPlayerModel = false;
        public bool IsolateCustomResources = true;
        public bool DisableAnnouncerVoices = true;
        public bool DisableApplauseSounds = true;
        public bool DisableDominationSounds = true;
        public bool EnhancedParticles = false;
        public readonly List<string> CustomResources = new List<string>();
        public readonly List<string> EnhancedParticleFiles = new List<string>();
    }

    internal sealed class HlaeRecordingSettingsForm : Form
    {
        private readonly TextBox hlaeBox = new TextBox();
        private readonly TextBox ffmpegBox = new TextBox();
        private readonly TextBox tf2Box = new TextBox();
        private readonly TextBox outputBox = new TextBox();
        private readonly TextBox lawenaResourcesBox = new TextBox();
        private readonly ComboBox resolution = DropDown();
        private readonly ComboBox dxLevel = DropDown();
        private readonly ComboBox skybox = DropDown();
        private readonly ComboBox hud = DropDown();
        private readonly ComboBox viewmodels = DropDown();
        private readonly NumericUpDown viewmodelFov = new NumericUpDown();
        private readonly CheckBox maximumGraphics = Check("Maximum-quality graphics profile");
        private readonly CheckBox motionBlur = Check("Enable motion blur");
        private readonly CheckBox disableHitSounds = Check("Disable hit sounds");
        private readonly CheckBox disableVoiceChat = Check("Disable voice chat");
        private readonly CheckBox minimalHud = Check("Minimal HUD");
        private readonly CheckBox disableCombatText = Check("Disable combat text");
        private readonly CheckBox disableCrosshair = Check("Disable crosshair");
        private readonly CheckBox disableCrosshairSwitching = Check("Disable crosshair switching");
        private readonly CheckBox hudPlayerModel = Check("3D player model in HUD");
        private readonly CheckBox isolateCustom = Check("Temporarily isolate custom resources");
        private readonly CheckBox announcer = Check("Disable announcer voices");
        private readonly CheckBox applause = Check("Disable applause sounds");
        private readonly CheckBox domination = Check("Disable domination/revenge sounds");
        private readonly CheckBox enhancedParticles = Check("Enable enhanced particles");
        private readonly CheckedListBox customResources = new CheckedListBox();
        private readonly Button particlesButton = new Button();
        private readonly List<string> selectedParticleFiles = new List<string>();
        private readonly HlaeRecordingSettings initial;

        public HlaeRecordingSettings Settings
        {
            get
            {
                HlaeRecordingSettings value = new HlaeRecordingSettings();
                value.FfmpegExecutable = ffmpegBox.Text.Trim();
                value.HlaeExecutable = hlaeBox.Text.Trim();
                value.Tf2Executable = tf2Box.Text.Trim();
                value.OutputDirectory = outputBox.Text.Trim();
                value.LawenaResourcesDirectory = lawenaResourcesBox.Text.Trim();
                value.Resolution = Convert.ToString(resolution.SelectedItem);
                value.DxLevel = Convert.ToString(dxLevel.SelectedItem);
                value.Skybox = Convert.ToString(skybox.SelectedItem);
                value.Hud = Convert.ToString(hud.SelectedItem);
                value.Viewmodels = Convert.ToString(viewmodels.SelectedItem);
                value.ViewmodelFov = (int)viewmodelFov.Value;
                value.MaximumGraphics = maximumGraphics.Checked;
                value.MotionBlur = motionBlur.Checked;
                value.DisableHitSounds = disableHitSounds.Checked;
                value.DisableVoiceChat = disableVoiceChat.Checked;
                value.MinimalHud = minimalHud.Checked;
                value.DisableCombatText = disableCombatText.Checked;
                value.DisableCrosshair = disableCrosshair.Checked;
                value.DisableCrosshairSwitching = disableCrosshairSwitching.Checked;
                value.HudPlayerModel = hudPlayerModel.Checked;
                value.IsolateCustomResources = isolateCustom.Checked;
                value.DisableAnnouncerVoices = announcer.Checked;
                value.DisableApplauseSounds = applause.Checked;
                value.DisableDominationSounds = domination.Checked;
                value.EnhancedParticles = enhancedParticles.Checked;
                foreach (object item in customResources.CheckedItems) value.CustomResources.Add(Convert.ToString(item));
                value.EnhancedParticleFiles.AddRange(selectedParticleFiles);
                return value;
            }
        }

        public HlaeRecordingSettingsForm(HlaeRecordingSettings source)
        {
            initial = source ?? new HlaeRecordingSettings();
            Text = "HLAE recording and movie settings (offline only)";
            StartPosition = FormStartPosition.CenterParent;
            MinimumSize = new Size(980, 690);
            Size = new Size(1060, 760);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;

            TableLayoutPanel root = new TableLayoutPanel();
            root.Dock = DockStyle.Fill;
            root.Padding = new Padding(12);
            root.RowCount = 3;
            root.ColumnCount = 1;
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 34));
            root.RowStyles.Add(new RowStyle(SizeType.Absolute, 45));
            Controls.Add(root);

            TabControl tabs = new TabControl();
            tabs.Dock = DockStyle.Fill;
            tabs.TabPages.Add(BuildPathsTab());
            tabs.TabPages.Add(BuildVisualsTab());
            tabs.TabPages.Add(BuildResourcesTab());
            root.Controls.Add(tabs, 0, 0);

            Label safety = new Label();
            safety.Text = "Recording is offline-only (-insecure, sv_lan 1). Your TF2 custom content, config.cfg, video.txt, movie profile, and DX level are backed up before launch and restored after TF2 exits or the parser closes.";
            safety.AutoSize = true;
            safety.ForeColor = Color.LightGreen;
            safety.Margin = new Padding(4, 8, 3, 2);
            root.Controls.Add(safety, 0, 1);

            FlowLayoutPanel buttons = new FlowLayoutPanel();
            buttons.Dock = DockStyle.Fill;
            buttons.FlowDirection = FlowDirection.RightToLeft;
            Button ok = new Button();
            ok.Text = "Prepare and launch";
            ok.Width = 150;
            ok.Height = 30;
            ok.DialogResult = DialogResult.OK;
            Button cancel = new Button();
            cancel.Text = "Cancel";
            cancel.Width = 90;
            cancel.Height = 30;
            cancel.DialogResult = DialogResult.Cancel;
            buttons.Controls.Add(ok);
            buttons.Controls.Add(cancel);
            root.Controls.Add(buttons, 0, 2);
            AcceptButton = ok;
            CancelButton = cancel;

            LoadValues();
        }

        private TabPage BuildPathsTab()
        {
            TabPage page = NewTab("Paths");
            TableLayoutPanel layout = NewGrid(6, 3);
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 175));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 105));
            page.Controls.Add(layout);
            AddPathRow(layout, 0, "FFmpeg.exe", ffmpegBox, BrowseFfmpeg);
            AddPathRow(layout, 1, "HLAE.exe", hlaeBox, BrowseHlae);
            AddPathRow(layout, 2, "TF2 executable", tf2Box, BrowseTf2);
            AddPathRow(layout, 3, "Recording output", outputBox, BrowseOutput);
            AddPathRow(layout, 4, "Lawena resources", lawenaResourcesBox, BrowseLawenaResources);
            Label note = NewLabel("Lawena resources provide the optional skyboxes, recording HUDs, sound suppressors, and PLDX enhanced particles included with this package.");
            note.ForeColor = Color.Silver;
            note.MaximumSize = new Size(700, 0);
            layout.SetColumnSpan(note, 2);
            layout.Controls.Add(note, 1, 5);
            return page;
        }

        private TabPage BuildVisualsTab()
        {
            TabPage page = NewTab("Video and HUD");
            TableLayoutPanel layout = NewGrid(8, 4);
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 145));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 145));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            page.Controls.Add(layout);

            AddChoice(layout, 0, 0, "Resolution", resolution, new string[] { "1280x720", "1920x1080", "2560x1440", "3840x2160" });
            AddChoice(layout, 0, 2, "DX level", dxLevel, new string[] { "Default", "98 (Lawena highest)", "95", "90", "81", "80" });
            AddChoice(layout, 1, 0, "Skybox", skybox, new string[] { "Default" });
            AddChoice(layout, 1, 2, "HUD", hud, new string[] { "Keep current", "Kill notices only", "Medic recording HUD", "Default TF2 HUD" });
            AddChoice(layout, 2, 0, "Viewmodels", viewmodels, new string[] { "On", "Off", "Default" });
            layout.Controls.Add(NewLabel("Viewmodel FOV"), 2, 2);
            viewmodelFov.Minimum = 1;
            viewmodelFov.Maximum = 179;
            viewmodelFov.Width = 80;
            layout.Controls.Add(viewmodelFov, 3, 2);

            GroupBox quality = NewGroup("Graphics");
            FlowLayoutPanel q = NewVerticalFlow();
            q.Controls.Add(maximumGraphics);
            q.Controls.Add(motionBlur);
            quality.Controls.Add(q);
            layout.SetColumnSpan(quality, 2);
            layout.Controls.Add(quality, 0, 3);

            GroupBox distractions = NewGroup("Additional settings");
            TableLayoutPanel checks = NewGrid(4, 2);
            checks.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            checks.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 50));
            checks.Controls.Add(disableHitSounds, 0, 0);
            checks.Controls.Add(disableCombatText, 1, 0);
            checks.Controls.Add(disableVoiceChat, 0, 1);
            checks.Controls.Add(disableCrosshair, 1, 1);
            checks.Controls.Add(minimalHud, 0, 2);
            checks.Controls.Add(disableCrosshairSwitching, 1, 2);
            checks.Controls.Add(hudPlayerModel, 0, 3);
            distractions.Controls.Add(checks);
            layout.SetColumnSpan(distractions, 4);
            layout.Controls.Add(distractions, 0, 4);

            Label dxNote = NewLabel("DX level is applied only to this HLAE launch. Default avoids the legacy override; 98 matches Lawena's displayed highest option.");
            dxNote.ForeColor = Color.Silver;
            layout.SetColumnSpan(dxNote, 4);
            layout.Controls.Add(dxNote, 0, 6);
            return page;
        }

        private TabPage BuildResourcesTab()
        {
            TabPage page = NewTab("Custom resources and particles");
            TableLayoutPanel layout = NewGrid(4, 2);
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 55));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 45));
            layout.RowStyles.Clear();
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 82));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 44));
            page.Controls.Add(layout);

            isolateCustom.AutoSize = true;
            isolateCustom.Margin = new Padding(4, 8, 4, 4);
            layout.SetColumnSpan(isolateCustom, 2);
            layout.Controls.Add(isolateCustom, 0, 0);

            GroupBox user = NewGroup("User custom resources to keep during recording");
            customResources.Dock = DockStyle.Fill;
            customResources.CheckOnClick = true;
            customResources.BackColor = Color.FromArgb(24, 26, 29);
            customResources.ForeColor = Color.Gainsboro;
            user.Controls.Add(customResources);
            layout.Controls.Add(user, 0, 1);

            GroupBox builtIns = NewGroup("Lawena recording resources");
            FlowLayoutPanel builtInFlow = NewVerticalFlow();
            builtInFlow.Controls.Add(announcer);
            builtInFlow.Controls.Add(applause);
            builtInFlow.Controls.Add(domination);
            builtInFlow.Controls.Add(enhancedParticles);
            particlesButton.Text = "Choose enhanced particle files...";
            particlesButton.Width = 230;
            particlesButton.Height = 30;
            particlesButton.Click += ChooseParticles;
            builtInFlow.Controls.Add(particlesButton);
            builtIns.Controls.Add(builtInFlow);
            layout.Controls.Add(builtIns, 1, 1);

            Label note = NewLabel("When isolation is enabled, the app moves your current tf/custom folder to a timestamped backup, creates a temporary recording folder containing only the checked resources, then moves your original folder back after TF2 closes.");
            note.ForeColor = Color.Silver;
            note.MaximumSize = new Size(850, 0);
            layout.SetColumnSpan(note, 2);
            layout.Controls.Add(note, 0, 2);

            FlowLayoutPanel actions = new FlowLayoutPanel();
            actions.Dock = DockStyle.Fill;
            Button refresh = SmallButton("Refresh list", delegate { PopulateCustomResources(); });
            Button all = SmallButton("All", delegate { SetAllCustomResources(true); });
            Button none = SmallButton("None", delegate { SetAllCustomResources(false); });
            actions.Controls.Add(refresh);
            actions.Controls.Add(all);
            actions.Controls.Add(none);
            layout.SetColumnSpan(actions, 2);
            layout.Controls.Add(actions, 0, 3);
            return page;
        }

        private void LoadValues()
        {
            ffmpegBox.Text = initial.FfmpegExecutable;
            hlaeBox.Text = initial.HlaeExecutable;
            tf2Box.Text = initial.Tf2Executable;
            outputBox.Text = initial.OutputDirectory;
            lawenaResourcesBox.Text = String.IsNullOrEmpty(initial.LawenaResourcesDirectory)
                ? RecordingProfileManager.FindLawenaResources() ?? ""
                : initial.LawenaResourcesDirectory;
            Select(resolution, initial.Resolution);
            Select(dxLevel, initial.DxLevel);
            Select(hud, initial.Hud);
            Select(viewmodels, initial.Viewmodels);
            viewmodelFov.Value = Math.Max(viewmodelFov.Minimum, Math.Min(viewmodelFov.Maximum, initial.ViewmodelFov));
            maximumGraphics.Checked = initial.MaximumGraphics;
            motionBlur.Checked = initial.MotionBlur;
            disableHitSounds.Checked = initial.DisableHitSounds;
            disableVoiceChat.Checked = initial.DisableVoiceChat;
            minimalHud.Checked = initial.MinimalHud;
            disableCombatText.Checked = initial.DisableCombatText;
            disableCrosshair.Checked = initial.DisableCrosshair;
            disableCrosshairSwitching.Checked = initial.DisableCrosshairSwitching;
            hudPlayerModel.Checked = initial.HudPlayerModel;
            isolateCustom.Checked = initial.IsolateCustomResources;
            announcer.Checked = initial.DisableAnnouncerVoices;
            applause.Checked = initial.DisableApplauseSounds;
            domination.Checked = initial.DisableDominationSounds;
            enhancedParticles.Checked = initial.EnhancedParticles;
            selectedParticleFiles.AddRange(initial.EnhancedParticleFiles);
            RefreshSkyboxes(initial.Skybox);
            PopulateCustomResources();
            particlesButton.Enabled = enhancedParticles.Checked;
            enhancedParticles.CheckedChanged += delegate { particlesButton.Enabled = enhancedParticles.Checked; };
        }

        private void PopulateCustomResources()
        {
            HashSet<string> checkedNames = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (object item in customResources.CheckedItems) checkedNames.Add(Convert.ToString(item));
            if (customResources.Items.Count == 0)
                foreach (string name in initial.CustomResources) checkedNames.Add(name);
            customResources.Items.Clear();
            string tfDirectory = TfGameDirectory(tf2Box.Text);
            string custom = String.IsNullOrEmpty(tfDirectory) ? "" : Path.Combine(tfDirectory, "custom");
            if (!Directory.Exists(custom)) return;
            List<string> names = new List<string>();
            foreach (string directory in Directory.GetDirectories(custom)) names.Add(Path.GetFileName(directory));
            foreach (string file in Directory.GetFiles(custom, "*.vpk")) names.Add(Path.GetFileName(file));
            names.Sort(StringComparer.OrdinalIgnoreCase);
            foreach (string name in names) customResources.Items.Add(name, checkedNames.Contains(name));
        }

        private void RefreshSkyboxes(string selected)
        {
            skybox.Items.Clear();
            skybox.Items.Add("Default");
            string folder = Path.Combine(lawenaResourcesBox.Text.Trim(), "skybox");
            if (Directory.Exists(folder))
            {
                List<string> names = new List<string>();
                foreach (string file in Directory.GetFiles(folder, "*up.vtf"))
                {
                    string name = Path.GetFileName(file);
                    names.Add(name.Substring(0, name.Length - "up.vtf".Length));
                }
                names.Sort(StringComparer.OrdinalIgnoreCase);
                foreach (string name in names) if (!skybox.Items.Contains(name)) skybox.Items.Add(name);
            }
            Select(skybox, selected);
        }

        private void ChooseParticles(object sender, EventArgs e)
        {
            using (EnhancedParticlesForm dialog = new EnhancedParticlesForm(selectedParticleFiles))
            {
                if (dialog.ShowDialog(this) != DialogResult.OK) return;
                selectedParticleFiles.Clear();
                selectedParticleFiles.AddRange(dialog.SelectedFiles);
                enhancedParticles.Checked = selectedParticleFiles.Count > 0;
            }
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
                if (dialog.ShowDialog(this) == DialogResult.OK)
                {
                    tf2Box.Text = dialog.FileName;
                    PopulateCustomResources();
                }
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

        private void BrowseLawenaResources(object sender, EventArgs e)
        {
            using (FolderBrowserDialog dialog = new FolderBrowserDialog())
            {
                dialog.Description = "Select the Lawena folder containing custom, hud, and skybox folders";
                if (Directory.Exists(lawenaResourcesBox.Text)) dialog.SelectedPath = lawenaResourcesBox.Text;
                if (dialog.ShowDialog(this) == DialogResult.OK)
                {
                    lawenaResourcesBox.Text = dialog.SelectedPath;
                    RefreshSkyboxes(Convert.ToString(skybox.SelectedItem));
                }
            }
        }

        private void SetAllCustomResources(bool selected)
        {
            for (int index = 0; index < customResources.Items.Count; index++) customResources.SetItemChecked(index, selected);
        }

        private static string TfGameDirectory(string executable)
        {
            if (!File.Exists(executable)) return null;
            string root = Path.GetDirectoryName(executable);
            string tf = Path.Combine(root, "tf");
            return Directory.Exists(tf) ? tf : null;
        }

        private static void AddPathRow(TableLayoutPanel layout, int row, string labelText, TextBox box, EventHandler browse)
        {
            layout.Controls.Add(NewLabel(labelText), 0, row);
            box.Dock = DockStyle.Fill;
            box.Margin = new Padding(3, 6, 3, 3);
            layout.Controls.Add(box, 1, row);
            Button button = new Button();
            button.Text = "Browse";
            button.Width = 95;
            button.Margin = new Padding(3, 5, 3, 3);
            button.Click += browse;
            layout.Controls.Add(button, 2, row);
        }

        private static void AddChoice(TableLayoutPanel layout, int row, int column, string label, ComboBox box, string[] values)
        {
            layout.Controls.Add(NewLabel(label), column, row);
            foreach (string value in values) box.Items.Add(value);
            box.Dock = DockStyle.Fill;
            box.Margin = new Padding(3, 5, 12, 3);
            layout.Controls.Add(box, column + 1, row);
        }

        private static void Select(ComboBox box, string value)
        {
            int index = box.Items.IndexOf(value);
            box.SelectedIndex = index >= 0 ? index : 0;
        }

        private static TabPage NewTab(string text)
        {
            TabPage page = new TabPage(text);
            page.BackColor = Color.FromArgb(30, 32, 36);
            page.ForeColor = Color.Gainsboro;
            page.Padding = new Padding(10);
            return page;
        }

        private static TableLayoutPanel NewGrid(int rows, int columns)
        {
            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.RowCount = rows;
            layout.ColumnCount = columns;
            layout.Padding = new Padding(6);
            return layout;
        }

        private static FlowLayoutPanel NewVerticalFlow()
        {
            FlowLayoutPanel panel = new FlowLayoutPanel();
            panel.Dock = DockStyle.Fill;
            panel.FlowDirection = FlowDirection.TopDown;
            panel.WrapContents = false;
            panel.AutoScroll = true;
            return panel;
        }

        private static GroupBox NewGroup(string text)
        {
            GroupBox group = new GroupBox();
            group.Text = text;
            group.Dock = DockStyle.Fill;
            group.ForeColor = Color.Gainsboro;
            group.Padding = new Padding(8);
            return group;
        }

        private static Button SmallButton(string text, EventHandler handler)
        {
            Button button = new Button();
            button.Text = text;
            button.Width = 95;
            button.Height = 30;
            button.Click += handler;
            return button;
        }

        private static ComboBox DropDown()
        {
            ComboBox box = new ComboBox();
            box.DropDownStyle = ComboBoxStyle.DropDownList;
            return box;
        }

        private static CheckBox Check(string text)
        {
            CheckBox box = new CheckBox();
            box.Text = text;
            box.AutoSize = true;
            box.Margin = new Padding(5, 5, 5, 3);
            return box;
        }

        private static Label NewLabel(string text)
        {
            Label label = new Label();
            label.Text = text;
            label.AutoSize = true;
            label.Margin = new Padding(3, 9, 3, 3);
            return label;
        }
    }

    internal sealed class EnhancedParticlesForm : Form
    {
        internal static readonly string[] ParticleFiles = new string[]
        {
            "particles/blood_impact.pcf", "particles/blood_trail.pcf", "particles/buildingdamage.pcf",
            "particles/bullet_tracers.pcf", "particles/burningplayer.pcf", "particles/cig_smoke.pcf",
            "particles/cinefx.pcf", "particles/class_fx.pcf", "particles/conc_stars.pcf", "particles/crit.pcf",
            "particles/dirty_explode.pcf", "particles/disguise.pcf", "particles/default.pcf",
            "particles/explosion.pcf", "particles/flag_particles.pcf", "particles/flamethrower.pcf",
            "particles/impact_fx.pcf", "particles/item_fx.pcf", "particles/medicgun_attrib.pcf",
            "particles/medicgun_beam.pcf", "particles/muzzle_flash.pcf", "particles/nailtrails.pcf",
            "particles/nemesis.pcf", "particles/player_recent_teleport.pcf", "particles/rocketbackblast.pcf",
            "particles/rocketjumptrail.pcf", "particles/rockettrail.pcf", "particles/scary_ghost.pcf",
            "particles/shellejection.pcf", "particles/smoke_blackbillow.pcf",
            "particles/smoke_blackbillow_hoodoo.pcf", "particles/soldierbuff.pcf", "particles/sparks.pcf",
            "particles/speechbubbles.pcf", "particles/stickybomb.pcf", "particles/teleport_status.pcf",
            "particles/teleported_fx.pcf", "particles/water.pcf"
        };

        private readonly CheckedListBox files = new CheckedListBox();
        public readonly List<string> SelectedFiles = new List<string>();

        public EnhancedParticlesForm(IList<string> selected)
        {
            Text = "Select Enhanced Particles";
            StartPosition = FormStartPosition.CenterParent;
            Size = new Size(520, 480);
            MinimumSize = new Size(460, 360);
            Font = new Font("Segoe UI", 9F);
            BackColor = Color.FromArgb(30, 32, 36);
            ForeColor = Color.Gainsboro;

            TableLayoutPanel layout = new TableLayoutPanel();
            layout.Dock = DockStyle.Fill;
            layout.Padding = new Padding(8);
            layout.RowCount = 3;
            layout.ColumnCount = 1;
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 32));
            layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));
            layout.RowStyles.Add(new RowStyle(SizeType.Absolute, 42));
            Controls.Add(layout);
            Label note = new Label();
            note.Text = "Select which PLDX enhanced particle files are copied into the temporary tf/custom folder.";
            note.AutoSize = true;
            layout.Controls.Add(note, 0, 0);
            files.Dock = DockStyle.Fill;
            files.CheckOnClick = true;
            files.BackColor = Color.FromArgb(24, 26, 29);
            files.ForeColor = Color.Gainsboro;
            HashSet<string> selectedSet = new HashSet<string>(selected ?? new List<string>(), StringComparer.OrdinalIgnoreCase);
            bool selectAll = selectedSet.Count == 0;
            foreach (string file in ParticleFiles) files.Items.Add(file, selectAll || selectedSet.Contains(file));
            layout.Controls.Add(files, 0, 1);

            FlowLayoutPanel buttons = new FlowLayoutPanel();
            buttons.Dock = DockStyle.Fill;
            Button all = new Button(); all.Text = "All"; all.Click += delegate { SetAll(true); };
            Button none = new Button(); none.Text = "None"; none.Click += delegate { SetAll(false); };
            Button ok = new Button(); ok.Text = "OK"; ok.DialogResult = DialogResult.OK; ok.Click += Save;
            Button cancel = new Button(); cancel.Text = "Cancel"; cancel.DialogResult = DialogResult.Cancel;
            buttons.Controls.Add(all); buttons.Controls.Add(none); buttons.Controls.Add(ok); buttons.Controls.Add(cancel);
            layout.Controls.Add(buttons, 0, 2);
            AcceptButton = ok;
            CancelButton = cancel;
        }

        private void SetAll(bool value)
        {
            for (int index = 0; index < files.Items.Count; index++) files.SetItemChecked(index, value);
        }

        private void Save(object sender, EventArgs e)
        {
            SelectedFiles.Clear();
            foreach (object item in files.CheckedItems) SelectedFiles.Add(Convert.ToString(item));
        }
    }

    internal sealed class RecordingProfileSession
    {
        public string SessionId;
        public string TfDirectory;
        public string BackupDirectory;
        public string OriginalCustomDirectory;
        public string TemporaryCustomDirectory;
        public string ProfileConfigPath;
        public string BackupProfileConfigPath;
        public bool OriginalCustomExisted;
        public bool ProfileConfigExisted;
        public bool IsolatedCustom;
        public readonly List<string> TemporaryRootFiles = new List<string>();
        public string OriginalProfileFolderDirectory;
        public bool OriginalProfileFolderExisted;
        public string VideoConfigPath;
        public string BackupVideoConfigPath;
        public bool VideoConfigExisted;
        public string ConfigPath;
        public string BackupConfigPath;
        public bool ConfigExisted;
        public bool DxLevelWasApplied;
        public bool OriginalDxLevelExisted;
        public object OriginalDxLevel;
        public int RestoreStarted;
    }

    internal static class RecordingProfileManager
    {
        private const string ProfileFolderName = "tf2fragdemohelper_recording";
        private const string ProfileConfigName = "tf2fragdemohelper_recording_profile.cfg";
        private static readonly object SessionLock = new object();
        private static RecordingProfileSession activeSession;

        public static string FindLawenaResources()
        {
            string app = AppDomain.CurrentDomain.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            string[] candidates = new string[]
            {
                Path.Combine(app, "lawena_resources"),
                Path.Combine(app, "lawena"),
                Path.Combine(Path.GetDirectoryName(app), "lawena_resources"),
                Path.Combine(Path.GetDirectoryName(app), "lawena")
            };
            foreach (string candidate in candidates)
                if (Directory.Exists(Path.Combine(candidate, "custom")) && Directory.Exists(Path.Combine(candidate, "skybox"))) return candidate;
            return null;
        }

        public static RecordingProfileSession Apply(string gameDirectory, string sessionId, HlaeRecordingSettings settings)
        {
            lock (SessionLock)
            {
                if (activeSession != null)
                    throw new InvalidOperationException("A recording profile is already active. Finish or close the current TF2 recording session first.");
            }
            if (File.Exists(ActivePointerPath())) RecoverInterruptedSession(null);
            if (File.Exists(ActivePointerPath()))
                throw new InvalidOperationException("A previous recording profile still needs recovery. Check active_recording_profile.json in Local AppData and the backup path recorded inside it before starting another recording.");
            RecordingProfileSession session = new RecordingProfileSession();
            session.SessionId = sessionId;
            session.TfDirectory = gameDirectory;
            session.BackupDirectory = Path.Combine(gameDirectory, "tf2fragdemohelper_backups", sessionId);
            session.OriginalCustomDirectory = Path.Combine(session.BackupDirectory, "custom_original");
            session.TemporaryCustomDirectory = Path.Combine(gameDirectory, "custom");
            session.ProfileConfigPath = Path.Combine(gameDirectory, "cfg", ProfileConfigName);
            session.BackupProfileConfigPath = Path.Combine(session.BackupDirectory, ProfileConfigName);
            session.OriginalProfileFolderDirectory = Path.Combine(session.BackupDirectory, "custom_profile_original");
            session.VideoConfigPath = Path.Combine(gameDirectory, "cfg", "video.txt");
            session.BackupVideoConfigPath = Path.Combine(session.BackupDirectory, "video.txt");
            session.ConfigPath = Path.Combine(gameDirectory, "cfg", "config.cfg");
            session.BackupConfigPath = Path.Combine(session.BackupDirectory, "config.cfg");
            session.IsolatedCustom = settings.IsolateCustomResources;
            Directory.CreateDirectory(session.BackupDirectory);
            WriteActivePointer(session, "preparing");

            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(session.ProfileConfigPath));
                if (File.Exists(session.ProfileConfigPath))
                {
                    session.ProfileConfigExisted = true;
                    File.Move(session.ProfileConfigPath, session.BackupProfileConfigPath);
                    WriteActivePointer(session, "preparing");
                }

                session.VideoConfigExisted = File.Exists(session.VideoConfigPath);
                if (session.VideoConfigExisted) File.Copy(session.VideoConfigPath, session.BackupVideoConfigPath, true);
                session.ConfigExisted = File.Exists(session.ConfigPath);
                if (session.ConfigExisted) File.Copy(session.ConfigPath, session.BackupConfigPath, true);
                CaptureDxLevel(session, settings.DxLevel);
                WriteActivePointer(session, "preparing");

                if (session.IsolatedCustom)
                {
                    session.OriginalCustomExisted = Directory.Exists(session.TemporaryCustomDirectory);
                    if (session.OriginalCustomExisted) Directory.Move(session.TemporaryCustomDirectory, session.OriginalCustomDirectory);
                    Directory.CreateDirectory(session.TemporaryCustomDirectory);
                    WriteActivePointer(session, "preparing");
                    foreach (string resource in settings.CustomResources)
                    {
                        string source = Path.Combine(session.OriginalCustomDirectory, Path.GetFileName(resource));
                        string destination = Path.Combine(session.TemporaryCustomDirectory, Path.GetFileName(resource));
                        if (Directory.Exists(source)) CopyDirectory(source, destination);
                        else if (File.Exists(source)) File.Copy(source, destination, true);
                    }
                }
                else
                {
                    Directory.CreateDirectory(session.TemporaryCustomDirectory);
                    string profileFolder = Path.Combine(session.TemporaryCustomDirectory, ProfileFolderName);
                    if (Directory.Exists(profileFolder))
                    {
                        session.OriginalProfileFolderExisted = true;
                        Directory.Move(profileFolder, session.OriginalProfileFolderDirectory);
                        WriteActivePointer(session, "preparing");
                    }
                }

                InstallLawenaResources(session, settings);
                File.WriteAllLines(session.ProfileConfigPath, BuildProfileConfig(settings).ToArray(), new UTF8Encoding(false));
                WriteActivePointer(session, "active");
                lock (SessionLock) activeSession = session;
                return session;
            }
            catch
            {
                Restore(session, false);
                throw;
            }
        }

        public static void Restore(RecordingProfileSession session, bool showErrors)
        {
            if (session == null || Interlocked.CompareExchange(ref session.RestoreStarted, 1, 0) != 0) return;
            try
            {
                if (session.IsolatedCustom)
                {
                    if (Directory.Exists(session.TemporaryCustomDirectory)) Directory.Delete(session.TemporaryCustomDirectory, true);
                    if (session.OriginalCustomExisted && Directory.Exists(session.OriginalCustomDirectory))
                        Directory.Move(session.OriginalCustomDirectory, session.TemporaryCustomDirectory);
                }
                else
                {
                    string profileFolder = Path.Combine(session.TemporaryCustomDirectory, ProfileFolderName);
                    if (Directory.Exists(profileFolder)) Directory.Delete(profileFolder, true);
                    foreach (string entry in session.TemporaryRootFiles)
                    {
                        string[] paths = entry.Split(new char[] { '|' }, 2);
                        if (File.Exists(paths[0])) File.Delete(paths[0]);
                        if (paths.Length == 2 && File.Exists(paths[1])) File.Move(paths[1], paths[0]);
                    }
                    if (session.OriginalProfileFolderExisted && Directory.Exists(session.OriginalProfileFolderDirectory))
                        Directory.Move(session.OriginalProfileFolderDirectory, profileFolder);
                }

                if (File.Exists(session.ProfileConfigPath)) File.Delete(session.ProfileConfigPath);
                if (session.ProfileConfigExisted && File.Exists(session.BackupProfileConfigPath))
                    File.Move(session.BackupProfileConfigPath, session.ProfileConfigPath);
                if (session.VideoConfigExisted && File.Exists(session.BackupVideoConfigPath))
                    File.Copy(session.BackupVideoConfigPath, session.VideoConfigPath, true);
                else if (!session.VideoConfigExisted && File.Exists(session.VideoConfigPath)) File.Delete(session.VideoConfigPath);
                if (File.Exists(session.BackupVideoConfigPath)) File.Delete(session.BackupVideoConfigPath);
                if (session.ConfigExisted && File.Exists(session.BackupConfigPath))
                    File.Copy(session.BackupConfigPath, session.ConfigPath, true);
                else if (!session.ConfigExisted && File.Exists(session.ConfigPath)) File.Delete(session.ConfigPath);
                if (File.Exists(session.BackupConfigPath)) File.Delete(session.BackupConfigPath);
                RestoreDxLevel(session);
                DeleteEmptyDirectory(session.BackupDirectory);
                DeleteEmptyDirectory(Path.GetDirectoryName(session.BackupDirectory));
                if (File.Exists(ActivePointerPath())) File.Delete(ActivePointerPath());
                lock (SessionLock) if (Object.ReferenceEquals(activeSession, session)) activeSession = null;
            }
            catch (Exception error)
            {
                session.RestoreStarted = 0;
                try { File.WriteAllText(Path.Combine(session.BackupDirectory, "RESTORE_REQUIRED.txt"), error.ToString(), new UTF8Encoding(false)); }
                catch { }
                if (showErrors)
                    MessageBox.Show("TF2 recording files could not be fully restored. Your original files remain in:\r\n" + session.BackupDirectory + "\r\n\r\n" + error.Message,
                        "TF2 Frag Demo Helper restore warning", MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
        }

        public static void RestoreActiveSession(bool showErrors)
        {
            RecordingProfileSession session;
            lock (SessionLock) session = activeSession;
            if (session != null) Restore(session, showErrors);
            else RecoverInterruptedSession(showErrors ? Form.ActiveForm : null);
        }

        public static void RecoverInterruptedSession(IWin32Window owner)
        {
            string pointer = ActivePointerPath();
            if (!File.Exists(pointer)) return;
            try
            {
                JavaScriptSerializer serializer = new JavaScriptSerializer();
                IDictionary values = serializer.DeserializeObject(File.ReadAllText(pointer)) as IDictionary;
                if (values == null) return;
                RecordingProfileSession session = new RecordingProfileSession();
                session.SessionId = Text(values, "session_id");
                session.TfDirectory = Text(values, "tf_directory");
                session.BackupDirectory = Text(values, "backup_directory");
                session.OriginalCustomDirectory = Text(values, "original_custom_directory");
                session.TemporaryCustomDirectory = Path.Combine(session.TfDirectory, "custom");
                session.ProfileConfigPath = Path.Combine(session.TfDirectory, "cfg", ProfileConfigName);
                session.BackupProfileConfigPath = Path.Combine(session.BackupDirectory, ProfileConfigName);
                session.OriginalProfileFolderDirectory = Path.Combine(session.BackupDirectory, "custom_profile_original");
                session.VideoConfigPath = Path.Combine(session.TfDirectory, "cfg", "video.txt");
                session.BackupVideoConfigPath = Path.Combine(session.BackupDirectory, "video.txt");
                session.ConfigPath = Path.Combine(session.TfDirectory, "cfg", "config.cfg");
                session.BackupConfigPath = Path.Combine(session.BackupDirectory, "config.cfg");
                session.OriginalCustomExisted = Bool(values, "original_custom_existed");
                session.ProfileConfigExisted = Bool(values, "profile_config_existed");
                session.IsolatedCustom = Bool(values, "isolated_custom");
                session.OriginalProfileFolderExisted = Bool(values, "original_profile_folder_existed");
                session.VideoConfigExisted = Bool(values, "video_config_existed");
                session.ConfigExisted = Bool(values, "config_existed");
                session.DxLevelWasApplied = Bool(values, "dx_level_was_applied");
                session.OriginalDxLevelExisted = Bool(values, "original_dx_level_existed");
                session.OriginalDxLevel = values.Contains("original_dx_level") ? values["original_dx_level"] : null;
                IList rootFiles = values.Contains("temporary_root_files") ? values["temporary_root_files"] as IList : null;
                if (rootFiles != null) foreach (object file in rootFiles) session.TemporaryRootFiles.Add(Convert.ToString(file));
                lock (SessionLock) activeSession = session;
                Restore(session, owner != null);
            }
            catch (Exception error)
            {
                if (owner != null)
                    MessageBox.Show(owner, "An interrupted recording profile needs manual recovery.\r\n\r\n" + error.Message,
                        "TF2 Frag Demo Helper restore warning", MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
        }

        private static void InstallLawenaResources(RecordingProfileSession session, HlaeRecordingSettings settings)
        {
            bool needsResources = settings.DisableAnnouncerVoices || settings.DisableApplauseSounds || settings.DisableDominationSounds ||
                settings.EnhancedParticles || !String.Equals(settings.Skybox, "Default", StringComparison.OrdinalIgnoreCase) ||
                !String.Equals(settings.Hud, "Keep current", StringComparison.OrdinalIgnoreCase) && !String.Equals(settings.Hud, "Default TF2 HUD", StringComparison.OrdinalIgnoreCase);
            if (!needsResources) return;
            string root = settings.LawenaResourcesDirectory;
            if (String.IsNullOrEmpty(root) || !Directory.Exists(root))
                throw new DirectoryNotFoundException("Select the Lawena resources folder. It must contain custom, hud, and skybox folders.");

            CopyOptionalVpk(session, root, "no_announcer_voices.vpk", settings.DisableAnnouncerVoices);
            CopyOptionalVpk(session, root, "no_applause_sounds.vpk", settings.DisableApplauseSounds);
            CopyOptionalVpk(session, root, "no_domination_sounds.vpk", settings.DisableDominationSounds);

            string profileRoot = Path.Combine(session.TemporaryCustomDirectory, ProfileFolderName);
            Directory.CreateDirectory(profileRoot);
            if (settings.EnhancedParticles)
            {
                string vpk = Path.Combine(root, "custom", "pldx_particles.vpk");
                if (!File.Exists(vpk)) throw new FileNotFoundException("Lawena's PLDX enhanced particle VPK was not found.", vpk);
                IList<string> selected = settings.EnhancedParticleFiles.Count == 0 ? (IList<string>)EnhancedParticlesForm.ParticleFiles : settings.EnhancedParticleFiles;
                ExtractParticleFiles(session.TfDirectory, vpk, profileRoot, selected);
            }

            if (!String.Equals(settings.Skybox, "Default", StringComparison.OrdinalIgnoreCase))
                InstallSkybox(root, profileRoot, settings.Skybox);
            if (String.Equals(settings.Hud, "Kill notices only", StringComparison.OrdinalIgnoreCase))
                CopyHud(root, profileRoot, "hud_killnotices");
            else if (String.Equals(settings.Hud, "Medic recording HUD", StringComparison.OrdinalIgnoreCase))
                CopyHud(root, profileRoot, "hud_medic");
        }

        private static void CopyOptionalVpk(RecordingProfileSession session, string resourcesRoot, string fileName, bool enabled)
        {
            if (!enabled) return;
            string source = Path.Combine(resourcesRoot, "custom", fileName);
            if (!File.Exists(source)) throw new FileNotFoundException("A selected Lawena sound resource was not found.", source);
            string destination = Path.Combine(session.TemporaryCustomDirectory, fileName);
            if (!session.IsolatedCustom && File.Exists(destination))
            {
                string backup = Path.Combine(session.BackupDirectory, "root_" + fileName);
                File.Move(destination, backup);
                session.TemporaryRootFiles.Add(destination + "|" + backup);
            }
            else session.TemporaryRootFiles.Add(destination);
            File.Copy(source, destination, true);
            WriteActivePointer(session, "preparing");
        }

        private static void InstallSkybox(string resourcesRoot, string profileRoot, string selected)
        {
            string source = Path.Combine(resourcesRoot, "skybox");
            string destination = Path.Combine(profileRoot, "materials", "skybox");
            Directory.CreateDirectory(destination);
            foreach (string vmt in Directory.GetFiles(source, "*.vmt")) File.Copy(vmt, Path.Combine(destination, Path.GetFileName(vmt)), true);
            string[] sides = new string[] { "bk", "dn", "ft", "lf", "rt", "up" };
            foreach (string side in sides)
            {
                string selectedVtf = Path.Combine(source, selected + side + ".vtf");
                if (!File.Exists(selectedVtf)) throw new FileNotFoundException("The selected Lawena skybox is incomplete.", selectedVtf);
                foreach (string vmt in Directory.GetFiles(destination, "*" + side + ".vmt"))
                    File.Copy(selectedVtf, Path.ChangeExtension(vmt, ".vtf"), true);
            }
        }

        private static void CopyHud(string resourcesRoot, string profileRoot, string hudName)
        {
            string source = Path.Combine(resourcesRoot, "hud", hudName);
            if (!Directory.Exists(source)) throw new DirectoryNotFoundException("The selected Lawena HUD was not found: " + source);
            CopyDirectory(source, profileRoot);
        }

        private static void ExtractParticleFiles(string tfDirectory, string vpk, string destination, IList<string> files)
        {
            string root = Directory.GetParent(tfDirectory).FullName;
            string[] tools = new string[]
            {
                Path.Combine(root, "bin", "vpk.exe"),
                Path.Combine(root, "bin", "x64", "vpk.exe"),
                Path.Combine(tfDirectory, "bin", "vpk.exe")
            };
            string vpkTool = null;
            foreach (string tool in tools) if (File.Exists(tool)) { vpkTool = tool; break; }
            if (vpkTool == null) throw new FileNotFoundException("TF2's vpk.exe is required to select enhanced particle files.");
            List<string> arguments = new List<string>();
            arguments.Add("x");
            arguments.Add(vpk);
            // Source discovers particle systems through this file.  It is part of the PLDX VPK
            // and must accompany even a hand-picked subset of PCFs.
            arguments.Add("particles/particles_manifest.txt");
            foreach (string file in files) arguments.Add(file.Replace('/', Path.DirectorySeparatorChar));
            ProcessStartInfo info = new ProcessStartInfo();
            info.FileName = vpkTool;
            info.Arguments = JoinArguments(arguments);
            info.WorkingDirectory = destination;
            info.UseShellExecute = false;
            info.CreateNoWindow = true;
            info.RedirectStandardError = true;
            info.RedirectStandardOutput = true;
            using (Process process = Process.Start(info))
            {
                string error = process.StandardError.ReadToEnd();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                if (process.ExitCode != 0)
                {
                    string detail = (error + "\r\n" + output).Trim();
                    if (String.IsNullOrEmpty(detail)) detail = "vpk.exe returned exit code " + process.ExitCode + ".";
                    throw new InvalidOperationException("Could not extract the selected enhanced particles. " + detail);
                }
            }
        }

        private static List<string> BuildProfileConfig(HlaeRecordingSettings settings)
        {
            List<string> lines = new List<string>();
            lines.Add("// Temporary movie profile generated by TF2 Frag Demo Helper.");
            lines.Add("sv_cheats 1");
            lines.Add("fps_max 0");
            if (settings.MaximumGraphics)
            {
                lines.AddRange(new string[]
                {
                    "cl_burninggibs 1", "cl_detaildist 8096", "cl_detailfade 0", "cl_maxrenderable_dist 8096",
                    "cl_new_impact_effects 1", "cl_phys_props_max 1024", "cl_ragdoll_collide 1", "lod_transitiondist 6400",
                    "mat_aaquality 2", "mat_antialias 8", "mat_bumpmap 1", "mat_compressedtextures 1",
                    "mat_envmapsize 512", "mat_envmaptgasize 512", "mat_forceaniso 16", "mat_hdr_level 2",
                    "mat_parallaxmap 1", "mat_picmip -1", "mat_postprocess_x 8", "mat_postprocess_y 8",
                    "mat_reducefillrate 0", "mat_software_aa_quality 2", "mat_software_aa_strength 2", "mat_specular 1",
                    "mat_vsync 0", "mat_wateroverlaysize 512", "mp_decals 4096", "mp_usehwmmodels 1", "mp_usehwmvcds 1",
                    "r_avglight 3", "r_decals 4096", "r_eyeglintlodpixels 4", "r_lod 0", "r_maxmodeldecal 4096",
                    "r_radiosity 3", "r_rainradius 2250", "r_rainsplashpercentage 100", "r_rootlod 0",
                    "r_shadowmaxrendered 1024", "r_shadowrendertotexture 1", "r_shadows 1", "r_waterdrawreflection 1",
                    "r_waterdrawrefraction 1", "r_waterforceexpensive 1", "r_waterforcereflectentities 1", "r_pixelfog 1",
                    "mat_viewportscale 1", "mat_viewportupscale 1", "mat_queue_mode -1", "r_threaded_particles 1",
                    "r_threaded_renderables 1", "r_threaded_client_shadow_manager 1"
                });
            }
            lines.Add("mat_motion_blur_enabled " + (settings.MotionBlur ? "1" : "0"));
            lines.Add("mat_motion_blur_forward_enabled " + (settings.MotionBlur ? "1" : "0"));
            lines.Add("mat_motion_blur_strength " + (settings.MotionBlur ? "1" : "0"));
            if (String.Equals(settings.Viewmodels, "On", StringComparison.OrdinalIgnoreCase)) lines.Add("r_drawviewmodel 1");
            else if (String.Equals(settings.Viewmodels, "Off", StringComparison.OrdinalIgnoreCase)) lines.Add("r_drawviewmodel 0");
            lines.Add("viewmodel_fov_demo " + settings.ViewmodelFov);
            lines.Add("hud_combattext " + (settings.DisableCombatText ? "0" : "1"));
            lines.Add("hud_combattext_healing " + (settings.DisableCombatText ? "0" : "1"));
            lines.Add("tf_dingalingaling " + (settings.DisableHitSounds ? "0" : "1"));
            lines.Add("tf_dingalingaling_lasthit " + (settings.DisableHitSounds ? "0" : "1"));
            lines.Add("voice_enable " + (settings.DisableVoiceChat ? "0" : "1"));
            lines.Add("cl_hud_minmode " + (settings.MinimalHud ? "1" : "0"));
            lines.Add("cl_hud_playerclass_playermodel_showed_confirm_dialog 1");
            lines.Add("cl_hud_playerclass_use_playermodel " + (settings.HudPlayerModel ? "1" : "0"));
            lines.Add("crosshair " + (settings.DisableCrosshair ? "0" : "1"));
            if (settings.DisableCrosshairSwitching)
            {
                lines.Add("alias cl_crosshair_file \"\"");
                lines.Add("alias cl_crosshair_scale \"\"");
                lines.Add("alias cl_crosshair_red \"\"");
                lines.Add("alias cl_crosshair_green \"\"");
                lines.Add("alias cl_crosshair_blue \"\"");
                lines.Add("alias crosshair \"\"");
            }
            lines.Add("cl_showfps 0");
            lines.Add("net_graph 0");
            lines.Add("hud_saytext_time 0");
            lines.Add("engine_no_focus_sleep 0");
            lines.Add("snd_mute_losefocus 0");
            lines.Add("echo TF2FRAG_MOVIE_PROFILE_READY");
            return lines;
        }

        private static void WriteActivePointer(RecordingProfileSession session, string state)
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            Dictionary<string, object> values = new Dictionary<string, object>();
            values["state"] = state;
            values["session_id"] = session.SessionId;
            values["tf_directory"] = session.TfDirectory;
            values["backup_directory"] = session.BackupDirectory;
            values["original_custom_directory"] = session.OriginalCustomDirectory;
            values["original_custom_existed"] = session.OriginalCustomExisted;
            values["profile_config_existed"] = session.ProfileConfigExisted;
            values["isolated_custom"] = session.IsolatedCustom;
            values["original_profile_folder_existed"] = session.OriginalProfileFolderExisted;
            values["video_config_existed"] = session.VideoConfigExisted;
            values["config_existed"] = session.ConfigExisted;
            values["dx_level_was_applied"] = session.DxLevelWasApplied;
            values["original_dx_level_existed"] = session.OriginalDxLevelExisted;
            values["original_dx_level"] = session.OriginalDxLevel;
            values["temporary_root_files"] = session.TemporaryRootFiles.ToArray();
            Directory.CreateDirectory(Path.GetDirectoryName(ActivePointerPath()));
            File.WriteAllText(ActivePointerPath(), serializer.Serialize(values), new UTF8Encoding(false));
        }

        private static string ActivePointerPath()
        {
            return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "TF2FragDemoHelper", "active_recording_profile.json");
        }

        private static void CopyDirectory(string source, string destination)
        {
            Directory.CreateDirectory(destination);
            foreach (string file in Directory.GetFiles(source)) File.Copy(file, Path.Combine(destination, Path.GetFileName(file)), true);
            foreach (string directory in Directory.GetDirectories(source)) CopyDirectory(directory, Path.Combine(destination, Path.GetFileName(directory)));
        }

        private static void CaptureDxLevel(RecordingProfileSession session, string selected)
        {
            session.DxLevelWasApplied = !String.IsNullOrEmpty(selected) && !selected.StartsWith("Default", StringComparison.OrdinalIgnoreCase);
            if (!session.DxLevelWasApplied) return;
            try
            {
                using (RegistryKey key = Registry.CurrentUser.OpenSubKey(@"Software\Valve\Source\tf\Settings", false))
                {
                    if (key == null) return;
                    session.OriginalDxLevel = key.GetValue("DXLevel_V1", null);
                    session.OriginalDxLevelExisted = session.OriginalDxLevel != null;
                }
            }
            catch { }
        }

        private static void RestoreDxLevel(RecordingProfileSession session)
        {
            if (!session.DxLevelWasApplied) return;
            try
            {
                using (RegistryKey key = Registry.CurrentUser.CreateSubKey(@"Software\Valve\Source\tf\Settings"))
                {
                    if (session.OriginalDxLevelExisted) key.SetValue("DXLevel_V1", session.OriginalDxLevel);
                    else key.DeleteValue("DXLevel_V1", false);
                }
            }
            catch { }
        }

        private static void DeleteEmptyDirectory(string path)
        {
            if (Directory.Exists(path) && Directory.GetFileSystemEntries(path).Length == 0) Directory.Delete(path);
        }

        private static string JoinArguments(IList<string> arguments)
        {
            StringBuilder result = new StringBuilder();
            foreach (string argument in arguments)
            {
                if (result.Length > 0) result.Append(' ');
                result.Append('"').Append((argument ?? "").Replace("\"", "\\\"")).Append('"');
            }
            return result.ToString();
        }

        private static string Text(IDictionary values, string key)
        {
            return values != null && values.Contains(key) && values[key] != null ? Convert.ToString(values[key]) : "";
        }

        private static bool Bool(IDictionary values, string key)
        {
            try { return values != null && values.Contains(key) && Convert.ToBoolean(values[key]); }
            catch { return false; }
        }
    }
}
