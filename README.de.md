# Teahouse · 茶馆 (Teehaus)

**Ein Ort, an dem kleine Gruppen (bis zu 15 Personen) gemeinsam Dinge erledigen.**

> Plaudern erzeugt Gefühle, ein Teehaus erzeugt Ergebnisse.

[English](README.md) · [中文](README.zh.md) · **Deutsch**

---

## Was es ist

Stell dir einen Gruppenchat vor, nur dass es hier ums Erledigen geht statt ums Plaudern:

- Jemand bringt ein **Problem** ein, etwa „Das Licht im Treppenhaus ist kaputt, wer repariert es?“ oder „Welches Buch liest der Lesekreis dieses Semester?“
- Alle schreiben **Ergebnisse** zu diesem Problem: Pläne, Dokumente, Tabellen, Fotos.
- Jedes Problem hat eine verantwortliche Person und einen Status (in Arbeit, gelöst), sodass man auf einen Blick sieht, wo es steht.
- Alles, was jemand geschrieben hat, bleibt erhalten. Wer es wann geschrieben hat, ist festgehalten, und niemand kann es nachträglich heimlich ändern.

Man kann es sich vorstellen als **einen Ordner, den alle teilen, plus ein paar Regeln**. Die Regeln bestimmen, wer direkt ändern darf, wer nur Änderungen vorschlagen kann und wer neue Leute hereinlassen darf.

## Warum es sich „nicht töten“ lässt

Gewöhnliche Gruppenchats und Cloud-Speicher liegen auf dem Server von jemand anderem. Verschwindet der Server oder wird ein Konto gesperrt, ist alles weg.

Ein Teehaus ist anders: **Auf dem Handy oder Computer jedes Mitglieds liegt eine vollständige Kopie.** Der Server, auf dem das Teehaus liegt (wir nennen ihn Relay), ist nur der Ort, an dem sich alle treffen und abgleichen. Verschwindet der Server, kann die Kopie jedes einzelnen Mitglieds das ganze Teehaus wiederherstellen, und der vom Wirt bestimmte „Gehilfe“ kann das Teehaus direkt übernehmen.

## Drei Rollen

| Rolle | Was sie darf | Man kann sie sich vorstellen als |
|---|---|---|
| **Wirt** (rotes Quadrat) | Alles ändern; entscheiden, wer einen Platz bekommt und welchen | Den Besitzer des Teehauses |
| **Gehilfe** (gelbes Dreieck) | Probleme, Ergebnisse und Aushänge direkt schreiben, Vorschläge anderer übernehmen; das Teehaus übernehmen, wenn der Wirt nicht mehr da ist | Die Vertrauensperson des Wirts und seinen Erben |
| **Gast** (blauer Kreis) | Alles lesen und mitdiskutieren; um Inhalte zu ändern, reicht er einen **Vorschlag** ein, den Wirt oder Gehilfe übernehmen | Einen Stammgast |

Auch eine KI kann Platz nehmen; neben ihrem Namen steht dann „AI“. Was sie darf, hängt wie bei einem Menschen allein vom Platz ab, den sie bekommt.

## Loslegen

**Jemand hat dich eingeladen:**
1. Lade die App herunter (siehe „Download“ unten).
2. Öffne sie, wähle „Einem Teehaus beitreten“ und füge die ganze Zeile der Einladung ein, die du bekommen hast (sie sieht aus wie `TH-XXXX-XXXX-XXXX-XXXX@Adresse`).
3. Fertig. Kein Konto, kein Passwort.

**Du willst selbst eines eröffnen:** Du brauchst einen Linux-Server (einer für ein paar Euro im Monat genügt). Installiere ihn nach [Ein Relay betreiben](docs/DEPLOY.md) und gib bei der Installation `--found` an; du bekommst eine einzeilige Gründungseinladung. Füge sie in die App ein, und du bist der Wirt.

**Du willst nur mal schauen:** Öffne die App und wähle „Erst die Beispiel-Teehäuser ansehen“. Es gibt sechs erfundene Teehäuser, alles Testdaten: nichts wird synchronisiert, nichts verlässt dein Gerät.

## Download

Neueste Testversion (wird nach jeder Codeänderung automatisch neu gebaut):

- **Android-Handy**: [teahouse-android-arm64.apk](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-android-arm64.apk)
- **Windows-Computer**: [teahouse-windows-x64-setup.exe](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-windows-x64-setup.exe)
- **Server (Linux)**: [teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz); siehe [Ein Relay betreiben](docs/DEPLOY.md)
- Alle Dateien und wann sie aktualisiert wurden: [Releases → nightly](https://github.com/letsEstel/teahouse/releases/tag/nightly)

Dies ist eine **frühe Testversion**: Funktionen kommen noch hinzu, und es kann Fehler geben. Bewahre Wichtiges zusätzlich an einem anderen Ort auf.

## Häufige Fragen

**Brauche ich ein Konto?** Nein. Dein Handy oder Computer erzeugt einen eigenen „Schlüssel“, der beweist, dass du du bist. Der Schlüssel bleibt auf deinem Gerät; du siehst ihn nie und musst dir kein Passwort merken.

**Können Handy und Computer dieselbe Person sein?** Ja. Tippe auf einem Gerät, das schon einen Platz hat, auf „Gerät hinzufügen“ und füge den Kopplungscode, den du bekommst, auf dem neuen Gerät ein. Bis zu 5 Geräte pro Person. (Der Bildschirm zum Koppeln ist noch in Arbeit.)

**Was, wenn ich mein Handy verliere?** Entferne es von einem deiner anderen Geräte aus; danach kommt es nie wieder hinein. Hast du alle verloren, bitte den Wirt, dich neu einzuladen.

**Kann der Betreiber des Servers lesen, was wir schreiben?** Ja. Teehaus verschlüsselt Inhalte noch nicht, deshalb sollte der Server in den Händen von jemandem liegen, dem ihr vertraut, am besten der eigene Server des Wirts.

**Warum höchstens 15 Personen?** Rund 15 ist die Grenze für eine kleine Gruppe, die wirklich gemeinsam etwas erledigt. Bei mehr Leuten ist es Zeit, sich auf mehrere Teehäuser aufzuteilen.

**Warum „Teehaus“?** Eine Hommage an Lao Shes Theaterstück *Das Teehaus*: ein Ort, an dem sich die verschiedensten Leute zusammensetzen und die Dinge bis zu einem Ergebnis besprechen.

## Dokumentation

Die folgenden Dokumente sind derzeit auf Chinesisch.

- [Benutzerhandbuch](docs/USER-GUIDE.md): für alle
- [Ein Relay betreiben](docs/DEPLOY.md): für alle, die ein eigenes Teehaus eröffnen wollen
- [Für Entwickler](docs/DEVELOP.md): Aufbau des Codes, Bauen, Testen, eine KI Platz nehmen lassen und alle Dokumente für Entwickler

## Lizenz

Teehaus wird unter der [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0-only) veröffentlicht. Alle dürfen es verwenden, verändern und weitergeben; wer aber eine veränderte Version anderen als Dienst anbietet (auch das Betreiben eines Relays für andere), muss den veränderten Quellcode ebenfalls seinen Nutzern zugänglich machen.
