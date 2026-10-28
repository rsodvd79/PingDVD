# PingDVD

Applicazione desktop in Rust per monitorare la latenza di rete con un grafico in tempo reale.

![Screenshot di PingDVD](screenshot.png)

## Funzionalità

- Ping continuo di hostname e indirizzi IP.
- Interfaccia scura con controlli adattivi, grafico con scala in ms, legenda e media tratteggiata.
- Storico degli ultimi 500 tentativi: risposte, timeout ed errori sono distinti; il grafico non collega le risposte attraverso i tentativi falliti.
- AVG, MIN e MAX calcolati sulle sole risposte valide dello storico; LAST mostra l'esito dell'ultimo tentativo. Nessun dato simulato all'avvio.
- Durata della sessione, congelata con Stop; una nuova sessione o un cambio host azzera storico e durata.
- Configurazione di host, timeout e intervallo (1-100000 ms), con Start/Stop, Apply e Reset.
- Salvataggio automatico delle impostazioni in `pingdvd.settings.json` accanto all'eseguibile; se non scrivibile, usa la cartella di configurazione utente.
- Esecuzione del comando `ping` del sistema operativo (`ping6` per indirizzi IPv6 su macOS), senza richiedere privilegi ICMP elevati.

## Utilizzo e impostazioni

Le modifiche valide a Host, Timeout e Interval hanno effetto immediato durante l'esecuzione. **Apply salva su disco**; non è necessario premerlo per applicare le modifiche live. **Reset** ripristina e salva i valori predefiniti.

Un host non valido viene segnalato senza sostituirlo automaticamente con Google: Start e Apply restano disabilitati. Durante una modifica non valida il monitoraggio continua sull'ultimo **Active host** mostrato nell'interfaccia. Al cambio di host valido parte una sessione pulita; la modifica di timeout o intervallo mantiene lo storico.

Timeout limita l'attesa complessiva di ciascun ping, inclusa la risoluzione del nome. Interval è la pausa **dopo** ogni tentativo, non il periodo fra gli inizi dei ping. L'asse orizzontale rappresenta i tentativi, non intervalli temporali uniformi. I segni arancioni/rossi indicano timeout/errori, non latenze.

Nelle finestre compatte le statistiche diventano una fascia testuale; se non resta spazio sufficiente per un grafico leggibile viene mostrato un invito ad allargare la finestra.

La configurazione viene cercata prima accanto all'eseguibile, poi nella cartella utente e infine nella directory corrente per compatibilità con installazioni precedenti. Il fallback utente è:

- macOS: `~/Library/Application Support/PingDVD/pingdvd.settings.json`
- Windows: `%APPDATA%\PingDVD\pingdvd.settings.json`

Le vecchie chiavi JSON `Host`, `Interval`, `TimeOut` e `Timeout` sono ancora leggibili. File malformati o non leggibili vengono segnalati e non sovrascritti; dopo un problema di caricamento il salvataggio automatico rimane disabilitato fino a un Apply riuscito. La scrittura usa un file temporaneo e sostituisce il precedente solo dopo aver completato il nuovo file.

## Requisiti e compilazione

È necessario Rust stable con Cargo.

```bash
cargo run
cargo test --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
```

Lo smoke test opzionale richiede i comandi ping di sistema e la rete di loopback, senza accesso a Internet:

```bash
cargo test --locked loopback_ipv4_and_ipv6_reply_and_worker_can_restart -- --ignored
```

Il workflow CI esegue formattazione, test, Clippy, smoke test IPv4/IPv6 e build per Windows e macOS. Su macOS il bundle `PingDVD.app` usa l'icona in `assets/pingdvd.icns`.
