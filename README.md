# PingDVD
GUI per il ping con grafico in tempo reale

Applicazione per il monitoraggio della latenza di rete costruita con Avalonia .NET che esegue il ping di un host specificato e visualizza i tempi di risposta in un grafico interattivo.

## Funzionalità

- **Monitoraggio Ping in Tempo Reale**: Esegue continuamente il ping di un host e visualizza la latenza nel tempo
- **Grafico Interattivo**: Polilinea DeepSkyBlue che mostra i tempi di ping con linea media OrangeRed
- **Indicatori di Stato**:
  - Indicatore LED (Rosso/Verde/Arancione) che mostra lo stato corrente
  - Testo di stato che visualizza "Fermato", "In Esecuzione" o messaggi di errore
  - Posizionato comodamente sotto il grafico
- **Gestione Migliorata degli Errori**:
  - Distingue tra host non raggiungibile, accesso negato e altri errori di ping
  - Fallback grazioso ai valori di timeout per continuare la visualizzazione
- **Controlli Utente**:
  - Campo per l'input dell'host (hostname o indirizzo IP)
  - Timeout configurabile (1-100000 ms)
  - Intervallo configurabile (1-100000 ms)
  - Pulsante Start/Stop per controllare le operazioni di ping
  - Pulsante Apply per salvare immediatamente le impostazioni
  - Pulsante Reset per ripristinare i valori predefiniti
- **Persistenza delle Impostazioni**:
  - Salvataggio automatico delle impostazioni alla chiusura dell'applicazione
  - Salvataggio manuale tramite pulsante Apply
  - Caricamento delle impostazioni salvate all'avvio
- **Miglioramenti al Grafico**:
  - Auto-scaling con range Y minimo (10ms) per la leggibilità quando i valori sono simili
  - Dimensione dello storico configurabile (predefinito 500 campioni)
  - Animazione fluida e aggiornamento continuo

## Dettagli Tecnici

- **Framework**: .NET 8.0 con Avalonia UI
- **Multi-piattaforma**: Principalmente testato su macOS, ma progettato per l'uso multi-piattaforma
- **Networking**: Utilizza la classe System.Net.NetworkInformation.Ping
- **UI**: Pattern ispirato al MVVM con logica nel code-behind
- **Persistenza**: Archiviazione delle impostazioni in formato JSON

## Impostazioni Predefinite

- **Host**: www.google.it
- **Intervallo**: 500 ms
- **Timeout**: 500 ms
- **Dimensione Storico**: 500 campioni
- **Grafico Iniziale**: Pre-popolato con 200 punti di campione per una visualizzazione immediata

## Miglioramenti Recenti

1. **Gestione Migliorata degli Errori**: Differenzia tra vari modi di guasto del ping
2. **Migliorato Feedback UI**: Indicatore LED di stato con messaggi contestuali
3. **Applicazione Immediata delle Impostazioni**: Pulsante Apply salva le impostazioni senza riavvio
4. **Reset con un Clic**: Ripristina istantaneamente i valori predefiniti
5. **Validazione dell'Input**: Validazione base hostname/IP prima dei tentativi di ping
6. **Leggibilità del Grafico**: Range Y minimo previene grafici illeggibili
7. **Qualità del Codice**: I magic numbers sostituiti con costanti denominate, calcoli ottimizzati

## Istruzioni per l'Uso

1. Avviare l'applicazione
2. Opzionalmente modificare i valori di Host, Timeout e Intervallo
3. Fare clic su "Start" per iniziare il ping
4. Osservare la latenza in tempo reale nel grafico
5. Utilizzare "Apply" per salvare le impostazioni correnti
6. Utilizzare "Reset" per tornare alle impostazioni di fabbrica
7. Fare clic su "Stop" per interrompere il pinging
8. Le impostazioni vengono salvate automaticamente all'uscita

## Come Compilare

```bash
dotnet build
dotnet run
```

## Licenza

Licenza MIT - vedere il file LICENSE per i dettagli.