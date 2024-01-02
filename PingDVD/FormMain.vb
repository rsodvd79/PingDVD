Imports System
Imports System.Configuration
Imports System.Net
Imports System.Net.NetworkInformation
Imports System.Text

Public Class FormMain

    Dim values As New List(Of Long)

    Public Sub New()
        ' La chiamata è richiesta dalla finestra di progettazione.
        InitializeComponent()

        ' Aggiungere le eventuali istruzioni di inizializzazione dopo la chiamata a InitializeComponent().
        ForeColor = Color.FromArgb(245, 246, 250)
        BackColor = Color.FromArgb(47, 54, 64)

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

    End Sub

    Private Sub ButtonStartStop_Click(sender As Object, e As EventArgs) Handles ButtonStartStop.Click

        TimerMain.Interval = CInt(NumericUpDownInterval.Value)

        ChartMain.ChartAreas(0).AxisY.Maximum = NumericUpDownTimeOut.Value

        TimerMain.Enabled = Not TimerMain.Enabled

    End Sub

    Private Sub TimerMain_Tick(sender As Object, e As EventArgs) Handles TimerMain.Tick
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
        ChartMain.SuspendLayout()

        ChartMain.Series(0).Points.Clear()
        ChartMain.Series(1).Points.Clear()

        For Each vl As Long In values
            ChartMain.Series(0).Points.AddY(vl)
            ChartMain.Series(1).Points.AddY(values.Average())
        Next

        Me.Text = $"PingDVD - AVG : {Math.Round(values.Average(), 2)} - LAST : {values.Last} - MIN : {values.Min} - MAX : {values.Max} (msec)"

        ChartMain.ResumeLayout()

    End Sub

    'Private Sub NumericUpDownInterval_ValueChanged(sender As Object, e As EventArgs) Handles NumericUpDownInterval.ValueChanged
    '    ConfigurationManager.AppSettings.Set("Interval", NumericUpDownInterval.Value.ToString(Globalization.CultureInfo.InvariantCulture))

    'End Sub

    'Private Sub NumericUpDownTimeOut_ValueChanged(sender As Object, e As EventArgs) Handles NumericUpDownTimeOut.ValueChanged
    '    ConfigurationManager.AppSettings.Set("TimeOut", NumericUpDownTimeOut.Value.ToString(Globalization.CultureInfo.InvariantCulture))

    'End Sub

    'Private Sub TextBoxHost_TextChanged(sender As Object, e As EventArgs) Handles TextBoxHost.TextChanged
    '    ConfigurationManager.AppSettings.Set("Host", TextBoxHost.Text)

    'End Sub


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
