﻿Imports System
Imports System.Net
Imports System.Net.NetworkInformation
Imports System.Text

Public Class FormMain

    Dim values As New List(Of Long)

    Private Sub ButtonStartStop_Click(sender As Object, e As EventArgs) Handles ButtonStartStop.Click
        If IsNumeric(TextBoxInterval.Text) Then
            TimerMain.Interval = Math.Max(1, CInt("0" & TextBoxInterval.Text))
        Else
            TimerMain.Interval = 1
        End If

        Dim timeout As Integer = 1

        If IsNumeric(TextBoxTimeout.Text) Then
            timeout = Math.Max(1, CInt("0" & TextBoxTimeout.Text))
        End If

        ChartMain.ChartAreas(0).AxisY.Maximum = timeout

        TimerMain.Enabled = Not TimerMain.Enabled

    End Sub

    Private Sub TimerMain_Tick(sender As Object, e As EventArgs) Handles TimerMain.Tick
        Try
            Dim options As New PingOptions With {.DontFragment = True}

            Dim buffer As Byte() = Encoding.ASCII.GetBytes(StrDup(32, Chr(0)))

            Dim pingSender As Ping = New Ping()

            Dim timeout As Integer = 1

            If IsNumeric(TextBoxTimeout.Text) Then
                timeout = Math.Max(1, CInt("0" & TextBoxTimeout.Text))
            End If

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

        Me.Text = $"PingDVD - AVG : {Math.Round(values.Average(), 2)} - LAST : {values.Last} - MIN : {values.Min} - MAX : {values.Max}"

        ChartMain.ResumeLayout()

    End Sub

End Class
