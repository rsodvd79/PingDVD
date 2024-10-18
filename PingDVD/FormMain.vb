Imports System
Imports System.Configuration
Imports System.Net
Imports System.Net.NetworkInformation
Imports System.Text

Public Class FormMain

    Dim values As New List(Of Long)
    Dim StartStop As Boolean = False

    Public Sub New()
        ' La chiamata è richiesta dalla finestra di progettazione.
        InitializeComponent()

        ' Aggiungere le eventuali istruzioni di inizializzazione dopo la chiamata a InitializeComponent().
        ForeColor = Color.FromArgb(230, 230, 230)
        BackColor = Color.FromArgb(30, 30, 30)

        TextBoxHost.ForeColor = ForeColor
        TextBoxHost.BackColor = BackColor

        NumericUpDownInterval.ForeColor = ForeColor
        NumericUpDownInterval.BackColor = BackColor

        NumericUpDownTimeOut.ForeColor = ForeColor
        NumericUpDownTimeOut.BackColor = BackColor

        ChartMain.ForeColor = ForeColor
        ChartMain.BackColor = BackColor

        ChartMain.ChartAreas(0).BackColor = BackColor

        ChartMain.ChartAreas(0).AxisY.InterlacedColor = ForeColor
        ChartMain.ChartAreas(0).AxisY.LabelStyle.ForeColor = ForeColor
        ChartMain.ChartAreas(0).AxisY.LineColor = ForeColor
        ChartMain.ChartAreas(0).AxisY.MajorTickMark.LineColor = ForeColor

        ChartMain.ChartAreas(0).AxisX.InterlacedColor = ForeColor
        ChartMain.ChartAreas(0).AxisX.LabelStyle.ForeColor = ForeColor
        ChartMain.ChartAreas(0).AxisX.LineColor = ForeColor
        ChartMain.ChartAreas(0).AxisX.MajorTickMark.LineColor = ForeColor

        ChartMain.Annotations.Clear()

        ChartMain.Annotations.Add(New DataVisualization.Charting.HorizontalLineAnnotation With {
            .AxisX = ChartMain.ChartAreas(0).AxisX,
            .AxisY = ChartMain.ChartAreas(0).AxisY,
            .ClipToChartArea = ChartMain.ChartAreas(0).Name,
            .IsInfinitive = True,
            .AnchorY = 0,
            .LineWidth = 2,
            .LineColor = Color.FromArgb(200, 252, 62, 54),
            .Visible = True
        })

        ChartMain.Series(0).Color = Color.FromArgb(255, 36, 92, 179)

    End Sub

    Private Sub ButtonStartStop_Click(sender As Object, e As EventArgs) Handles ButtonStartStop.Click

        TimerMain.Interval = CInt(NumericUpDownInterval.Value)

        ChartMain.ChartAreas(0).AxisY.Maximum = NumericUpDownTimeOut.Value

        StartStop = Not StartStop

        TimerMain.Enabled = StartStop

    End Sub

    Private Sub TimerMain_Tick(sender As Object, e As EventArgs) Handles TimerMain.Tick

        TimerMain.Stop()

        Task.Factory.StartNew(
                      Sub()
                          Try
                              Dim options As New PingOptions With {.DontFragment = True}

                              Dim buffer As Byte() = Encoding.ASCII.GetBytes(StrDup(32, Chr(0)))

                              Dim pingSender As Ping = New Ping()

                              Dim timeout As Integer = CInt(NumericUpDownTimeOut.Value)

                              Dim reply As PingReply = pingSender.Send(TextBoxHost.Text, timeout, buffer, options)

                              If reply.Status = IPStatus.Success Then
                                  values.Add(reply.RoundtripTime)
                              Else
                                  values.Add(timeout)
                              End If

                          Catch ex As Exception
                              values.Add(-1)

                          End Try

                          If values.Count > 500 Then
                              values.RemoveAt(0)
                          End If

                          LoadPoint()

                          TimerMain_Start()

                      End Sub
                    )

    End Sub

    Private Sub TimerMain_Start()
        If InvokeRequired Then
            BeginInvoke(New Action(AddressOf TimerMain_Start))
            Exit Sub
        End If

        If StartStop Then
            TimerMain.Start()
        End If

    End Sub

    Private Sub FormMain_Load(sender As Object, e As EventArgs) Handles Me.Load
        Dim conf As Configuration = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None)
        Dim sett As KeyValueConfigurationCollection = conf.AppSettings.Settings

        If sett("Interval") Is Nothing Then
            sett.Add("Interval", NumericUpDownInterval.Value.ToString(Globalization.CultureInfo.InvariantCulture))
        Else
            NumericUpDownInterval.Value = CDec(sett("Interval").Value)
        End If

        If sett("TimeOut") Is Nothing Then
            sett.Add("TimeOut", NumericUpDownTimeOut.Value.ToString(Globalization.CultureInfo.InvariantCulture))
        Else
            NumericUpDownTimeOut.Value = CDec(sett("TimeOut").Value)
        End If

        If sett("Host") Is Nothing Then
            sett.Add("Host", TextBoxHost.Text)
        Else
            TextBoxHost.Text = sett("Host").Value
        End If

        Dim lv As Long = 11

        For vl As Long = 1 To 500
            values.Add(Math.Max(lv, 9))
            lv -= 1
        Next

        LoadPoint()

    End Sub

    Private Sub LoadPoint()
        If InvokeRequired Then
            Invoke(New Action(AddressOf LoadPoint))
            Exit Sub
        End If

        ChartMain.SuspendLayout()

        ChartMain.Series(0).Points.Clear()

        For Each vl As Long In values
            ChartMain.Series(0).Points.AddY(vl)
        Next

        ChartMain.Annotations(0).AnchorY = values.Average()

        ChartMain.ResumeLayout()

        Me.Text = $"PingDVD - AVG : {Math.Round(values.Average(), 2)} msec - LAST : {values.Last} msec - MIN : {values.Min} msec - MAX : {values.Max} msec - {Date.MinValue.AddMilliseconds(NumericUpDownInterval.Value * values.Count):HH:mm:ss}"

    End Sub

    Private Sub FormMain_FormClosing(sender As Object, e As FormClosingEventArgs) Handles Me.FormClosing
        Dim conf As Configuration = ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None)
        Dim sett As KeyValueConfigurationCollection = conf.AppSettings.Settings

        If sett("Interval") Is Nothing Then
            sett.Add("Interval", NumericUpDownInterval.Value.ToString(Globalization.CultureInfo.InvariantCulture))
        Else
            sett("Interval").Value = NumericUpDownInterval.Value.ToString(Globalization.CultureInfo.InvariantCulture)
        End If

        If sett("TimeOut") Is Nothing Then
            sett.Add("TimeOut", NumericUpDownTimeOut.Value.ToString(Globalization.CultureInfo.InvariantCulture))
        Else
            sett("TimeOut").Value = NumericUpDownTimeOut.Value.ToString(Globalization.CultureInfo.InvariantCulture)
        End If

        If sett("Host") Is Nothing Then
            sett.Add("Host", TextBoxHost.Text)
        Else
            sett("Host").Value = TextBoxHost.Text
        End If

        conf.Save(ConfigurationSaveMode.Modified)
        ConfigurationManager.RefreshSection(conf.AppSettings.SectionInformation.Name)

    End Sub

End Class
