# Teahouse · Teehaus

*Architecture No. 1 in Rust, Op. 1*

v0.1 · *Anji Baicha* (安吉白茶)

[English](README.md) · [中文](README.zh.md) · **Deutsch**

## Vorwort

Damit eine Gruppe etwas zustande bringt, braucht sie drei Dinge: eine Liste der Probleme, die alle sehen können, einen Ort, an dem die Ergebnisse aufbewahrt werden, und Regeln, die vorher vereinbart sind. Ein Gruppenchat bietet keines davon.

Das Teehaus bietet alle drei. Ein Teehaus ist ein gemeinsames Verzeichnis mit höchstens fünfzehn Plätzen. Es gibt drei Arten von Plätzen: Wirt, Gehilfe und Gast. Jeder Schreibvorgang muss sich auf ein Problem beziehen und vom Gerät der schreibenden Person signiert sein. Wer direkt schreiben darf und wer nur vorschlagen kann, bestimmt der Platz; durchgesetzt wird es vom Server.

Auf dem Gerät jedes Mitglieds liegt die vollständige Geschichte des Teehauses. Der Server kann abgeschaltet werden; das Teehaus verschwindet nicht mit ihm.

### Plätze

Der **Wirt** darf alles ändern, auch die Sitzordnung: wer im Teehaus sitzt und auf welchem Platz.

Der **Gehilfe** schreibt Probleme, Ergebnisse und Aushänge direkt und nimmt Vorschläge anderer an oder lehnt sie ab. Der Gehilfe ist auch der Erbe: Ist der Server des Wirts verschwunden, eröffnet der Gehilfe das Teehaus aus seiner eigenen Kopie neu.

Ein **Gast** liest alles und nimmt an der Diskussion teil. Um Inhalte hinzuzufügen oder zu ändern, reicht ein Gast einen Vorschlag ein, den Wirt oder Gehilfe annehmen oder ablehnen. Beide Ausgänge bleiben in der Geschichte.

Einen Platz kann ein Mensch oder eine KI einnehmen. Eine KI ist überall, wo sie erscheint, als solche gekennzeichnet; was sie darf, bestimmt wie bei einem Menschen ihr Platz.

### Grundsätze

- **Nichts wird umgeschrieben.** Die Geschichte wächst nur. Jede Änderung ist ein neuer Eintrag; niemand, auch nicht der Wirt, kann einen früheren ändern oder tilgen.
- **Befugnis kommt aus der Sitzordnung.** Das Relay beurteilt jeden Schreibvorgang nach der Sitzordnung, wie sie **vor** ihm gilt; kein Schreibvorgang kann daher seinen eigenen Urheber befördern.
- **Die Identität bleibt auf dem Gerät.** Jedes Gerät erzeugt seinen eigenen Schlüssel, und der Schlüssel verlässt es nie. Es gibt keine Konten und keine Passwörter.
- **Fünfzehn Plätze.** Fünfzehn ist etwa die größte Gruppe, die noch gemeinsam handeln kann. Eine größere Gruppe sollte mehrere Teehäuser bilden.

## Stand

Das Teehaus befindet sich in einer frühen Testphase. Es läuft auf Android und Windows, der Server (das Relay) auf Linux. Inhalte sind noch nicht verschlüsselt: Wer das Relay betreibt, kann sie lesen; das Relay sollte daher jemandem gehören, dem die Mitglieder vertrauen. Bewahre Wichtiges zusätzlich an einem anderen Ort auf.

## Erste Schritte

**Einem Teehaus beitreten:** Installiere die App, wähle *Einem Teehaus beitreten* und füge die ganze Einladung ein, die du erhalten hast (`TH-XXXX-XXXX-XXXX-XXXX@Adresse`).

**Ein Teehaus eröffnen:** Installiere ein Relay auf einem Linux-Server, wie in [Ein Relay betreiben](docs/DEPLOY.md) beschrieben, und gib dabei `--found` an. Das Installationsprogramm gibt eine Gründungseinladung aus; wer sie einlöst, wird Wirt.

**Erst schauen:** Wähle in der App *Erst die Beispiel-Teehäuser ansehen*. Die Beispiele sind erfunden und bleiben auf deinem Gerät.

## Download

Automatisch aus dem neuesten Code gebaut:

- Android: [teahouse-android-arm64.apk](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-android-arm64.apk)
- Windows: [teahouse-windows-x64-setup.exe](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-windows-x64-setup.exe)
- Relay (Linux-Server): [teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz)
- Alle Dateien: [Releases → nightly](https://github.com/letsEstel/teahouse/releases/tag/nightly)

## Dokumentation

Die folgenden Dokumente sind derzeit auf Chinesisch.

- [Benutzerhandbuch](docs/USER-GUIDE.md): ein Teehaus benutzen, und Antworten auf häufige Fragen
- [Ein Relay betreiben](docs/DEPLOY.md): ein Teehaus eröffnen und pflegen
- [Für Entwickler](docs/DEVELOP.md): der Code, die Gesetze, die das Relay durchsetzt, und die Schnittstellen

## Lizenz

Das Teehaus wird unter der [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0-only) veröffentlicht. Alle dürfen es verwenden, verändern und weitergeben. Wer eine veränderte Version anderen als Dienst anbietet, auch durch den Betrieb eines Relays, muss den veränderten Quellcode deren Nutzern zugänglich machen.
