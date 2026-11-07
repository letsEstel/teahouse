# Das Gesamtbild: von einem Problem zu dreißigtausend Zeilen

[English](OVERVIEW.md) · [中文](OVERVIEW.zh.md) · **Deutsch**

Am leichtesten verirrt man sich in einem großen Projekt, wenn man in der ersten Zeile der ersten Datei anfängt. Code ist eine Antwort, und eine Antwort versteht nur, wer die Frage kennt. Diese Seite geht von oben nach unten: wofür Teahouse da ist, welche Entscheidungen daraus folgen, und erst dann der Code.

## Erste Ebene: das Problem

Eine kleine Gruppe will etwas erledigen, zum Beispiel die Parkplätze in einer Wohnanlage regeln. Meistens hat sie einen Gruppenchat. Drei Monate später stehen darin Zehntausende Nachrichten, und er beantwortet die drei einfachsten Fragen nicht:

1. Was ist zu tun, und wie weit ist es?
2. Wo liegen die Ergebnisse, und welche Fassung ist die neueste?
3. Wer entscheidet, und wer darf was ändern?

Ein Gruppenchat ist eine Zeitleiste. Neue Nachrichten schieben alte weg, nichts bleibt liegen. Der Chat ist nicht schlecht gemacht; eine Zeitleiste ist für diese Aufgabe nur die falsche Form.

Teahouse gibt der Gruppe stattdessen drei Dinge: eine Liste der Probleme, die alle sehen, einen Ort für die Ergebnisse und Regeln, die vorab vereinbart sind und automatisch gelten.

## Zweite Ebene: die Entscheidungen

Die ersten beiden sind leicht. Die dritte ist schwer. Nimmt man „Regeln, die automatisch gelten“ ernst, folgt daraus eine Kette von Entscheidungen.

**Regeln brauchen jemanden, der sie durchsetzt, also braucht es einen Schiedsrichter.** Stehen die Regeln nur in der Software der einzelnen Leute, umgeht man sie, indem man die Software ändert. Geurteilt werden muss an einem Ort außerhalb der Hände der Nutzer. Dieser Ort heißt **Relay**.

**Aber der Schiedsrichter darf kein einzelner Ausfallpunkt werden.** Läge alles nur auf dem Relay, wäre das Teehaus mit ihm verschwunden, und wer es betreibt, hätte alle in der Hand. Deshalb hält jedes Gerät jedes Mitglieds eine vollständige Kopie, und das Relay ist nur der Treffpunkt der Kopien. Der Server kann verschwinden, das Teehaus bleibt.

**Viele Kopien bedeuten Synchronisation und Geschichte.** Genau dieses Problem hat git gelöst. Ein Teehaus ist ein git-Repository, ein Schreibvorgang ist ein Commit. Dazu bringt git eine weitere Eigenschaft mit: Jeder Commit trägt als Namen einen Hash seines Inhalts, und dieser Inhalt enthält den Hash des vorherigen Commits. Ändert man etwas in der Vergangenheit, ändern sich alle späteren Namen. Die Geschichte lässt sich nicht heimlich umschreiben, und das beruht auf Mathematik, nicht auf einem Versprechen.

**Der Schiedsrichter muss wissen, wer wer ist.** Konten und Passwörter bedeuten eine Passwortdatenbank, das Begehrteste, was ein Dienst besitzen kann. Stattdessen erzeugt jedes Gerät sein eigenes Schlüsselpaar. Der private Schlüssel verlässt das Gerät nie, der öffentliche steht im Sitzplan. Der Sitzplan ist selbst eine Datei im Repository, also hat auch die Frage, wer was darf, eine vollständige Geschichte.

**Die Regeln dürfen sich nicht selbst ändern können.** Könnte ein Push den Sitzplan ändern und dann nach dem geänderten Sitzplan beurteilt werden, könnte sich jeder selbst befördern. Deshalb urteilt das Relay immer nach dem Sitzplan, wie er **vor** dem Push war.

Diese fünf Entscheidungen greifen ineinander. Nimmt man eine weg, muss man die anderen neu durchdenken. Im Code begegnen sie einem immer wieder.

Eine Zahl verdient eine eigene Bemerkung: **fünfzehn Plätze**. Die Grenze hat nichts mit Leistung zu tun; git trägt Projekte wie den Linux-Kernel mit Tausenden von Beteiligten. Fünfzehn ist eine organisatorische Entscheidung. Darüber beginnt eine Gruppe, Arbeit aufzuteilen, Sitzungen abzuhalten und Ebenen zu bilden, und ist keine Gruppe mehr, die gemeinsam eine Sache erledigt. Darum ist die Zahl ein Gesetz, das das Relay durchsetzt. Wird eine Aufgabe größer, ist die Antwort ein weiteres Teehaus, kein größeres.

## Dritte Ebene: nichts neu erfinden

In einem Sicherheitssystem ist der gefährlichste Code der selbst geschriebene. Teahouse steht, so weit es geht, auf Dingen, die Jahrzehnte von Angriffen überstanden haben:

| Bedarf | Was es liefert | Unter Beschuss seit |
|---|---|---|
| Authentifizierung und verschlüsselter Transport | OpenSSH | 1999 |
| Speicherung, Synchronisation, Geschichte | git | 2005 |
| Abfangen und Urteilen | git-Hooks | 2005 |
| Signaturen | ed25519 | 2011 |

Das Relay hat deshalb keinen eigenen Netzwerkdienst und parst kein Netzwerkprotokoll. Es klinkt sich an zwei Stellen ein: OpenSSH ruft es auf, nachdem ein Schlüssel bestätigt ist, und git ruft es auf, bevor Commits angenommen werden. Es tut das Einzige, was noch niemand getan hatte: **nach Sitzplatz urteilen**.

Für diese Denkweise gibt es eine klassische Arbeit: *The Protection of Information in Computer Systems* von Saltzer und Schroeder (1975). Ihre Prinzipien gelten bis heute, vier davon nutzt Teahouse:

- **Einfacher Mechanismus** (economy of mechanism): Je weniger Code zu prüfen ist, desto besser. Das Relay nutzt keine Bibliotheken von Dritten; jede Zeile, die entscheidet, wer hereinkommt, steht in diesem Repository.
- **Sichere Voreinstellung** (fail-safe defaults): Im Zweifel ablehnen.
- **Vollständige Kontrolle** (complete mediation): Jeder Commit und jeder Pfad wird jedes Mal geprüft. Niemand wird durchgewinkt, weil er eben schon geprüft wurde.
- **Geringste Rechte** (least privilege): Ein Gast darf nur eigene Beiträge hinzufügen; ein Gehilfe kommt nicht an den Sitzplan.

## Vierte Ebene: vier Teile

```text
  ┌───────────── Gerät eines Mitglieds ────────────┐
  │                                                │
  │   app/        Oberfläche  Svelte + TypeScript  │
  │     │ ruft auf                                 │
  │   client/     Engine      Rust: Kopie, Signatur│
  │     │                                          │
  └─────┼──────────────────────────────────────────┘
        │ ssh + git
  ┌─────┼──────────────── Server ──────────────────┐
  │     ▼                                          │
  │   sshd        OpenSSH, nicht von uns           │
  │     │                                          │
  │   relay/      Torwächter  Rust, ohne Abhängigk.│
  │     │                                          │
  │   core/       Regeln      Rust, rein, ohne I/O │
  └────────────────────────────────────────────────┘
```

`core/` wird von `relay/` und `client/` benutzt; es auf der Serverseite zu zeichnen, ist also nicht ganz genau: Es ist der Gesetzestext, den beide teilen.

| Verzeichnis | Etwa Zeilen | Hängt ab von | Was es ist |
|---|---|---|---|
| `core/` | 2.500 | nichts | Sitzplan, Schlüssel, Regeln |
| `relay/` | 7.000 | nur `core` | der Torwächter auf dem Server |
| `client/` | 11.000 | libgit2 und andere | die Engine auf jedem Gerät |
| `app/` | 8.500 | Svelte, Tauri | die Oberfläche, Hüllen für Handy und Desktop |

Die Abhängigkeiten zeigen in eine Richtung: Äußeres hängt von Innerem ab, `core/` von nichts. Diese Richtung bestimmt die Reihenfolge beim Lesen und zeigt, wo ein Fehler am teuersten ist.

## Fünfte Ebene: das Leben eines Beitrags

Nun alle Teile zusammen. Tante Wang schreibt unter Problem 0001 einen Beitrag und tippt auf Senden.

1. Die **Oberfläche** gibt den Text mit einem einzigen Befehl, `note`, an die Engine. Von git und ssh weiß sie nichts.
2. Die **Engine** schreibt eine Datei `talk/0001/<Zeit>-wang.md` in die lokale Kopie, macht einen Commit mit `Problem: 0001` in der Nachricht und signiert ihn mit dem privaten Schlüssel dieses Geräts.
3. Der Beitrag ist jetzt auf ihrem Gerät, auch ohne Netz geht er nicht verloren. Die Engine versucht anschließend einen Push.
4. **ssh** verbindet sich mit dem Server. sshd bestätigt mit ihrem öffentlichen Schlüssel, dass sie den privaten besitzt, und lässt dann nichts anderes laufen als `teahouse-relay shell <Schlüssel>`.
5. Die **Shell des Relays** schaut in den Sitzplan: Hat dieser Schlüssel in diesem Teehaus einen Platz? Wenn ja, ersetzt sie sich selbst durch `git receive-pack`.
6. **git** legt den neuen Commit in Quarantäne, lässt `main` unverändert und führt den Hook `pre-receive` aus.
7. Der **Hook des Relays** prüft den Commit. Stimmt die Signatur? Ist sie die Autorin? Gibt es eine Zeile `Problem:`? Existiert dieses Problem? Sie ist Gast: Fügt die Änderung nur einen eigenen Beitrag hinzu? Liegt der Zeitstempel mehr als zehn Minuten vor der Serverzeit?
8. Geht alles durch, bewegt git `main`. Scheitert etwas, ist der ganze Push nichtig, die Engine erhält den Grund, und die Oberfläche zeigt ihn an.
9. Beim nächsten Synchronisieren erhalten die anderen den Commit. Von da an liegt er auf allen Geräten, mit Inhalt, Signatur und seinem Platz in der Geschichte.

Von diesen neun Schritten kommt unser Code in 1, 2, 3, 5, 7 und 9 vor. Der Rest stammt von anderen: älterer Code, weit öfter angegriffen.

## Weiter

- Teahouse benutzen: das [Benutzerhandbuch](https://github.com/letsEstel/teahouse/blob/main/docs/USER-GUIDE.md) (auf Chinesisch).
- Den Code lesen: [für Entwickler](https://github.com/letsEstel/teahouse/blob/main/docs/DEVELOP.md) (auf Chinesisch). Beginne mit `core/`: Es ist der kleinste Teil und der wichtigste.

Wenn du vergisst, wozu ein Stück Code dient, komm auf diese Seite zurück und suche seinen Platz im Diagramm.
