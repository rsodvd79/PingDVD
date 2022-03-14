Imports System
Imports System.Net
Imports System.Net.NetworkInformation
Imports System.Text

Public Class FormMain

    Dim values As New List(Of Long)

    Private Sub Button1_Click(sender As Object, e As EventArgs) Handles Button1.Click
        TimerMain.Interval = Math.Max(1, CInt("0" & TextBoxInterval.Text))
        TimerMain.Enabled = Not TimerMain.Enabled

    End Sub

    Private Sub TimerMain_Tick(sender As Object, e As EventArgs) Handles TimerMain.Tick
        Try
            Dim options As New PingOptions With {.DontFragment = True}

            Dim buffer As Byte() = Encoding.ASCII.GetBytes(StrDup(32, Chr(0)))

            Dim pingSender As Ping = New Ping()
            Dim reply As PingReply = pingSender.Send(TextBoxHost.Text, Math.Max(1, CInt("0" & TextBoxTimeout.Text)), buffer, options)

            If reply.Status = IPStatus.Success Then
                values.Add(reply.RoundtripTime)
            Else
                values.Add(0)
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

        For Each vl As Long In values
            ChartMain.Series(0).Points.AddY(vl)
        Next

        ChartMain.ResumeLayout()

    End Sub

End Class
