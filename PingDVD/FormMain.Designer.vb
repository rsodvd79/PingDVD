<Global.Microsoft.VisualBasic.CompilerServices.DesignerGenerated()> _
Partial Class FormMain
    Inherits System.Windows.Forms.Form

    'Form esegue l'override del metodo Dispose per pulire l'elenco dei componenti.
    <System.Diagnostics.DebuggerNonUserCode()> _
    Protected Overrides Sub Dispose(ByVal disposing As Boolean)
        Try
            If disposing AndAlso components IsNot Nothing Then
                components.Dispose()
            End If
        Finally
            MyBase.Dispose(disposing)
        End Try
    End Sub

    'Richiesto da Progettazione Windows Form
    Private components As System.ComponentModel.IContainer

    'NOTA: la procedura che segue è richiesta da Progettazione Windows Form
    'Può essere modificata in Progettazione Windows Form.  
    'Non modificarla mediante l'editor del codice.
    <System.Diagnostics.DebuggerStepThrough()> _
    Private Sub InitializeComponent()
        Me.components = New System.ComponentModel.Container()
        Dim ChartArea1 As System.Windows.Forms.DataVisualization.Charting.ChartArea = New System.Windows.Forms.DataVisualization.Charting.ChartArea()
        Dim Series1 As System.Windows.Forms.DataVisualization.Charting.Series = New System.Windows.Forms.DataVisualization.Charting.Series()
        Dim resources As System.ComponentModel.ComponentResourceManager = New System.ComponentModel.ComponentResourceManager(GetType(FormMain))
        Me.ChartMain = New System.Windows.Forms.DataVisualization.Charting.Chart()
        Me.TimerMain = New System.Windows.Forms.Timer(Me.components)
        Me.TableLayoutPanel1 = New System.Windows.Forms.TableLayoutPanel()
        Me.TableLayoutPanel2 = New System.Windows.Forms.TableLayoutPanel()
        Me.Label1 = New System.Windows.Forms.Label()
        Me.TextBoxHost = New System.Windows.Forms.TextBox()
        Me.Label2 = New System.Windows.Forms.Label()
        Me.Label3 = New System.Windows.Forms.Label()
        Me.ButtonStartStop = New System.Windows.Forms.Button()
        Me.NumericUpDownTimeOut = New System.Windows.Forms.NumericUpDown()
        Me.NumericUpDownInterval = New System.Windows.Forms.NumericUpDown()
        CType(Me.ChartMain, System.ComponentModel.ISupportInitialize).BeginInit()
        Me.TableLayoutPanel1.SuspendLayout()
        Me.TableLayoutPanel2.SuspendLayout()
        CType(Me.NumericUpDownTimeOut, System.ComponentModel.ISupportInitialize).BeginInit()
        CType(Me.NumericUpDownInterval, System.ComponentModel.ISupportInitialize).BeginInit()
        Me.SuspendLayout()
        '
        'ChartMain
        '
        Me.ChartMain.BackColor = System.Drawing.SystemColors.Control
        ChartArea1.AxisX.IntervalAutoMode = System.Windows.Forms.DataVisualization.Charting.IntervalAutoMode.VariableCount
        ChartArea1.AxisX.IsMarginVisible = False
        ChartArea1.AxisX.IsStartedFromZero = False
        ChartArea1.AxisX.LabelStyle.Enabled = False
        ChartArea1.AxisX.MajorGrid.Enabled = False
        ChartArea1.AxisX.MajorGrid.LineColor = System.Drawing.Color.Silver
        ChartArea1.AxisX.MajorTickMark.Enabled = False
        ChartArea1.AxisX.MajorTickMark.Interval = 0R
        ChartArea1.AxisY.IntervalAutoMode = System.Windows.Forms.DataVisualization.Charting.IntervalAutoMode.VariableCount
        ChartArea1.AxisY.IsMarginVisible = False
        ChartArea1.AxisY.IsStartedFromZero = False
        ChartArea1.AxisY.MajorGrid.LineColor = System.Drawing.Color.Silver
        ChartArea1.AxisY.Minimum = 0R
        ChartArea1.AxisY.MinorGrid.LineColor = System.Drawing.Color.Silver
        ChartArea1.Name = "ChartArea1"
        Me.ChartMain.ChartAreas.Add(ChartArea1)
        Me.ChartMain.Dock = System.Windows.Forms.DockStyle.Fill
        Me.ChartMain.Location = New System.Drawing.Point(3, 29)
        Me.ChartMain.Name = "ChartMain"
        Series1.BorderWidth = 3
        Series1.ChartArea = "ChartArea1"
        Series1.ChartType = System.Windows.Forms.DataVisualization.Charting.SeriesChartType.Line
        Series1.Name = "SeriesPing"
        Me.ChartMain.Series.Add(Series1)
        Me.ChartMain.Size = New System.Drawing.Size(794, 418)
        Me.ChartMain.TabIndex = 1
        Me.ChartMain.Text = "Chart1"
        '
        'TimerMain
        '
        Me.TimerMain.Interval = 500
        '
        'TableLayoutPanel1
        '
        Me.TableLayoutPanel1.ColumnCount = 1
        Me.TableLayoutPanel1.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle())
        Me.TableLayoutPanel1.Controls.Add(Me.ChartMain, 0, 1)
        Me.TableLayoutPanel1.Controls.Add(Me.TableLayoutPanel2, 0, 0)
        Me.TableLayoutPanel1.Dock = System.Windows.Forms.DockStyle.Fill
        Me.TableLayoutPanel1.Location = New System.Drawing.Point(0, 0)
        Me.TableLayoutPanel1.Name = "TableLayoutPanel1"
        Me.TableLayoutPanel1.RowCount = 2
        Me.TableLayoutPanel1.RowStyles.Add(New System.Windows.Forms.RowStyle())
        Me.TableLayoutPanel1.RowStyles.Add(New System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Percent, 100.0!))
        Me.TableLayoutPanel1.Size = New System.Drawing.Size(800, 450)
        Me.TableLayoutPanel1.TabIndex = 2
        '
        'TableLayoutPanel2
        '
        Me.TableLayoutPanel2.ColumnCount = 8
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle())
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 25.0!))
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle())
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 25.0!))
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle())
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 25.0!))
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle())
        Me.TableLayoutPanel2.ColumnStyles.Add(New System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 25.0!))
        Me.TableLayoutPanel2.Controls.Add(Me.Label1, 0, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.TextBoxHost, 1, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.Label2, 2, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.Label3, 4, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.ButtonStartStop, 7, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.NumericUpDownTimeOut, 3, 0)
        Me.TableLayoutPanel2.Controls.Add(Me.NumericUpDownInterval, 5, 0)
        Me.TableLayoutPanel2.Dock = System.Windows.Forms.DockStyle.Top
        Me.TableLayoutPanel2.Location = New System.Drawing.Point(0, 0)
        Me.TableLayoutPanel2.Margin = New System.Windows.Forms.Padding(0)
        Me.TableLayoutPanel2.Name = "TableLayoutPanel2"
        Me.TableLayoutPanel2.RowCount = 1
        Me.TableLayoutPanel2.RowStyles.Add(New System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Percent, 100.0!))
        Me.TableLayoutPanel2.Size = New System.Drawing.Size(800, 26)
        Me.TableLayoutPanel2.TabIndex = 2
        '
        'Label1
        '
        Me.Label1.AutoSize = True
        Me.Label1.Dock = System.Windows.Forms.DockStyle.Right
        Me.Label1.Location = New System.Drawing.Point(3, 0)
        Me.Label1.Name = "Label1"
        Me.Label1.Size = New System.Drawing.Size(29, 26)
        Me.Label1.TabIndex = 0
        Me.Label1.Text = "Host"
        Me.Label1.TextAlign = System.Drawing.ContentAlignment.MiddleRight
        '
        'TextBoxHost
        '
        Me.TextBoxHost.BorderStyle = System.Windows.Forms.BorderStyle.FixedSingle
        Me.TextBoxHost.Dock = System.Windows.Forms.DockStyle.Top
        Me.TextBoxHost.Location = New System.Drawing.Point(38, 3)
        Me.TextBoxHost.Name = "TextBoxHost"
        Me.TextBoxHost.Size = New System.Drawing.Size(160, 20)
        Me.TextBoxHost.TabIndex = 1
        Me.TextBoxHost.Text = "www.google.it"
        '
        'Label2
        '
        Me.Label2.AutoSize = True
        Me.Label2.Dock = System.Windows.Forms.DockStyle.Right
        Me.Label2.Location = New System.Drawing.Point(204, 0)
        Me.Label2.Name = "Label2"
        Me.Label2.Size = New System.Drawing.Size(45, 26)
        Me.Label2.TabIndex = 2
        Me.Label2.Text = "Timeout"
        Me.Label2.TextAlign = System.Drawing.ContentAlignment.MiddleRight
        '
        'Label3
        '
        Me.Label3.AutoSize = True
        Me.Label3.Dock = System.Windows.Forms.DockStyle.Right
        Me.Label3.Location = New System.Drawing.Point(421, 0)
        Me.Label3.Name = "Label3"
        Me.Label3.Size = New System.Drawing.Size(42, 26)
        Me.Label3.TabIndex = 4
        Me.Label3.Text = "Interval"
        Me.Label3.TextAlign = System.Drawing.ContentAlignment.MiddleRight
        '
        'ButtonStartStop
        '
        Me.ButtonStartStop.Dock = System.Windows.Forms.DockStyle.Top
        Me.ButtonStartStop.FlatStyle = System.Windows.Forms.FlatStyle.Flat
        Me.ButtonStartStop.Location = New System.Drawing.Point(632, 0)
        Me.ButtonStartStop.Margin = New System.Windows.Forms.Padding(0, 0, 3, 0)
        Me.ButtonStartStop.Name = "ButtonStartStop"
        Me.ButtonStartStop.Size = New System.Drawing.Size(165, 23)
        Me.ButtonStartStop.TabIndex = 6
        Me.ButtonStartStop.Text = "Start\Stop"
        Me.ButtonStartStop.UseVisualStyleBackColor = True
        '
        'NumericUpDownTimeOut
        '
        Me.NumericUpDownTimeOut.Dock = System.Windows.Forms.DockStyle.Top
        Me.NumericUpDownTimeOut.Location = New System.Drawing.Point(255, 3)
        Me.NumericUpDownTimeOut.Maximum = New Decimal(New Integer() {100000, 0, 0, 0})
        Me.NumericUpDownTimeOut.Minimum = New Decimal(New Integer() {1, 0, 0, 0})
        Me.NumericUpDownTimeOut.Name = "NumericUpDownTimeOut"
        Me.NumericUpDownTimeOut.Size = New System.Drawing.Size(160, 20)
        Me.NumericUpDownTimeOut.TabIndex = 7
        Me.NumericUpDownTimeOut.Value = New Decimal(New Integer() {500, 0, 0, 0})
        '
        'NumericUpDownInterval
        '
        Me.NumericUpDownInterval.Dock = System.Windows.Forms.DockStyle.Top
        Me.NumericUpDownInterval.Location = New System.Drawing.Point(469, 3)
        Me.NumericUpDownInterval.Maximum = New Decimal(New Integer() {100000, 0, 0, 0})
        Me.NumericUpDownInterval.Minimum = New Decimal(New Integer() {1, 0, 0, 0})
        Me.NumericUpDownInterval.Name = "NumericUpDownInterval"
        Me.NumericUpDownInterval.Size = New System.Drawing.Size(160, 20)
        Me.NumericUpDownInterval.TabIndex = 8
        Me.NumericUpDownInterval.Value = New Decimal(New Integer() {500, 0, 0, 0})
        '
        'FormMain
        '
        Me.AutoScaleDimensions = New System.Drawing.SizeF(6.0!, 13.0!)
        Me.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font
        Me.BackColor = System.Drawing.SystemColors.Control
        Me.ClientSize = New System.Drawing.Size(800, 450)
        Me.Controls.Add(Me.TableLayoutPanel1)
        Me.ForeColor = System.Drawing.SystemColors.ControlText
        Me.Icon = CType(resources.GetObject("$this.Icon"), System.Drawing.Icon)
        Me.Name = "FormMain"
        Me.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen
        Me.Text = "PingDVD"
        CType(Me.ChartMain, System.ComponentModel.ISupportInitialize).EndInit()
        Me.TableLayoutPanel1.ResumeLayout(False)
        Me.TableLayoutPanel2.ResumeLayout(False)
        Me.TableLayoutPanel2.PerformLayout()
        CType(Me.NumericUpDownTimeOut, System.ComponentModel.ISupportInitialize).EndInit()
        CType(Me.NumericUpDownInterval, System.ComponentModel.ISupportInitialize).EndInit()
        Me.ResumeLayout(False)

    End Sub
    Friend WithEvents ChartMain As Chart
    Friend WithEvents TimerMain As Timer
    Friend WithEvents TableLayoutPanel1 As TableLayoutPanel
    Friend WithEvents TableLayoutPanel2 As TableLayoutPanel
    Friend WithEvents Label1 As Label
    Friend WithEvents TextBoxHost As TextBox
    Friend WithEvents Label2 As Label
    Friend WithEvents Label3 As Label
    Friend WithEvents ButtonStartStop As Button
    Friend WithEvents NumericUpDownTimeOut As NumericUpDown
    Friend WithEvents NumericUpDownInterval As NumericUpDown
End Class
