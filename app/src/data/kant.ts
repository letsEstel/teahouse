// 康德读书会：六个人每周读《纯粹理性批判》，读不懂的地方写成问题。当前用户是茶客。
import type { House } from './types'

const glossary = { zh: '术语对照表', en: 'Glossary of terms', de: 'Begriffsverzeichnis' }
const translateDing = { zh: '术语表：Ding an sich 统一译为“物自身”', en: 'Glossary: always render Ding an sich as 物自身', de: 'Glossar: Ding an sich durchgehend als 物自身 übersetzen' }

export const kant: House = {
  id: 'kant',
  name: { zh: '康德读书会', en: 'Kant Reading Circle', de: 'Kant-Lesekreis' },
  nameLines: { zh: ['康德', '读书会'], en: ['Kant', 'Reading Circle'], de: ['Kant-', 'Lesekreis'] },
  sign: {
    zh: '每周四晚，一次读十页。读不懂的地方，写成问题。',
    en: 'Thursday nights, ten pages at a time. Whatever we can’t follow becomes a problem.',
    de: 'Donnerstagabends, zehn Seiten pro Treffen. Was wir nicht verstehen, wird zum Problem.',
  },
  founded: '2026-01-08',
  capacity: 15,
  me: 'xu',
  members: [
    { id: 'su', name: { zh: '苏明哲', en: 'Su Mingzhe', de: 'Su Mingzhe' }, role: 'host', joined: '2026-01-08' },
    { id: 'he', name: { zh: '何雨', en: 'He Yu', de: 'He Yu' }, role: 'steward', joined: '2026-01-08' },
    { id: 'xu', name: { zh: '许清', en: 'Xu Qing', de: 'Xu Qing' }, role: 'guest', joined: '2026-01-15' },
    { id: 'luo', name: { zh: '罗森', en: 'Luo Sen', de: 'Luo Sen' }, role: 'guest', joined: '2026-01-15' },
    { id: 'tang', name: { zh: '唐棠', en: 'Tang Tang', de: 'Tang Tang' }, role: 'guest', joined: '2026-02-05' },
    { id: 'ge', name: { zh: '葛文', en: 'Ge Wen', de: 'Ge Wen' }, role: 'guest', joined: '2026-04-30' },
  ],
  problems: [
    {
      id: '0001',
      title: {
        zh: '先验演绎到底证明了什么',
        en: 'What does the Transcendental Deduction actually prove?',
        de: 'Was beweist die transzendentale Deduktion eigentlich?',
      },
      summary: {
        zh: 'A 版和 B 版的论证路线不同，读了三遍还是说不清它们各自证明了什么。',
        en: 'The A and B editions argue differently. Three readings in, we still can’t say what each one proves.',
        de: 'A- und B-Auflage argumentieren verschieden. Nach drei Durchgängen können wir nicht sagen, was jede beweist.',
      },
      status: 'active',
      by: 'luo',
      opened: '2026-09-04',
      updated: '2026-10-16T22:40',
      body: [
        {
          heading: { zh: '卡在哪里', en: 'Where we’re stuck', de: 'Wo wir hängen' },
          text: {
            zh: 'B 版 §20 说范畴对一切直观有效，§26 又重新证明一遍。两节是重复，还是两步论证？',
            en: 'B §20 says the categories hold for all intuition; §26 proves it again. Repetition, or two steps of one argument?',
            de: 'B §20 sagt, die Kategorien gelten für alle Anschauung; §26 beweist es noch einmal. Wiederholung oder zwei Schritte eines Arguments?',
          },
        },
        {
          heading: { zh: '想弄清', en: 'What we want to settle', de: 'Was wir klären wollen' },
          text: {
            zh: '能用一页纸写出 B 版演绎的论证结构，每个人都同意。',
            en: 'One page laying out the structure of the B Deduction that everyone agrees with.',
            de: 'Eine Seite, die den Aufbau der B-Deduktion darlegt und der alle zustimmen.',
          },
        },
        {
          heading: { zh: '读法', en: 'How we read', de: 'Wie wir lesen' },
          text: {
            zh: '以李秋零译本为准，对照德文原文；二手文献只用来检验，不替代原文。',
            en: 'Li Qiuling’s translation against the German original; secondary literature only to check, never to replace.',
            de: 'Li Qiulings Übersetzung neben dem Original; Sekundärliteratur nur zur Prüfung, nie als Ersatz.',
          },
        },
      ],
      works: [
        {
          id: 'k1', kind: 'plan', by: 'he', when: '2026-10-16T22:40',
          title: { zh: 'B 版演绎论证结构（第三稿）', en: 'Structure of the B Deduction (3rd draft)', de: 'Aufbau der B-Deduktion (3. Fassung)' },
          note: { zh: '§15–§27，分两步', en: '§15–§27, in two steps', de: '§15–§27, in zwei Schritten' },
          preview: {
            excerpt: {
              zh: '第一步（§15–§20）：凡是被统觉统一起来的直观，都服从范畴。\n第二步（§21–§26）：我们的一切感性直观，都能被统觉统一。\n所以：范畴对一切经验对象有效。',
              en: 'Step one (§15–§20): any intuition unified by apperception is subject to the categories.\nStep two (§21–§26): all our sensible intuition can be unified by apperception.\nTherefore: the categories hold for every object of experience.',
              de: 'Erster Schritt (§15–§20): Jede durch Apperzeption vereinigte Anschauung steht unter den Kategorien.\nZweiter Schritt (§21–§26): Alle unsere sinnliche Anschauung ist durch Apperzeption vereinbar.\nAlso: Die Kategorien gelten für alle Gegenstände der Erfahrung.',
            },
          },
        },
        {
          id: 'k2', kind: 'doc', by: 'he', when: '2026-10-09T23:00', title: glossary,
          note: { zh: '中德对照，41 条', en: 'Chinese–German, 41 entries', de: 'Chinesisch–Deutsch, 41 Einträge' },
          history: [
            {
              by: 'he', when: '2026-09-20T22:00', act: 'add',
              diff: [
                { kind: 'add', text: { zh: 'Anschauung — 直观', en: 'Anschauung — 直观 (intuition)', de: 'Anschauung — 直观' } },
                { kind: 'add', text: { zh: 'Erscheinung — 显象', en: 'Erscheinung — 显象 (appearance)', de: 'Erscheinung — 显象' } },
                { kind: 'add', text: { zh: 'Ding an sich — 物自体', en: 'Ding an sich — 物自体 (thing in itself)', de: 'Ding an sich — 物自体' } },
              ],
            },
            {
              by: 'he', when: '2026-10-09T23:00', act: 'update',
              diff: [
                { kind: 'same', text: { zh: 'Anschauung — 直观', en: 'Anschauung — 直观 (intuition)', de: 'Anschauung — 直观' } },
                { kind: 'same', text: { zh: 'Erscheinung — 显象', en: 'Erscheinung — 显象 (appearance)', de: 'Erscheinung — 显象' } },
                { kind: 'same', text: { zh: 'Ding an sich — 物自体', en: 'Ding an sich — 物自体 (thing in itself)', de: 'Ding an sich — 物自体' } },
                { kind: 'add', text: { zh: 'Noumenon — 本体、物自体', en: 'Noumenon — 本体, 物自体', de: 'Noumenon — 本体, 物自体' } },
                { kind: 'add', text: { zh: 'Verstand — 知性', en: 'Verstand — 知性 (understanding)', de: 'Verstand — 知性' } },
              ],
            },
          ],
          lines: [
            { zh: 'Anschauung — 直观', en: 'Anschauung — 直观 (intuition)', de: 'Anschauung — 直观' },
            { zh: 'Erscheinung — 显象', en: 'Erscheinung — 显象 (appearance)', de: 'Erscheinung — 显象' },
            { zh: 'Ding an sich — 物自体', en: 'Ding an sich — 物自体 (thing in itself)', de: 'Ding an sich — 物自体' },
            { zh: 'Noumenon — 本体、物自体', en: 'Noumenon — 本体, 物自体', de: 'Noumenon — 本体, 物自体' },
            { zh: 'Verstand — 知性', en: 'Verstand — 知性 (understanding)', de: 'Verstand — 知性' },
          ],
        },
        {
          id: 'k3', kind: 'doc', by: 'tang', when: '2026-10-02T22:30',
          title: { zh: '第三次讨论纪要', en: 'Notes from the third session', de: 'Protokoll der dritten Sitzung' },
        },
        {
          id: 'k4', kind: 'doc', by: 'ge', when: '2026-09-25T20:00',
          title: { zh: '二手文献清单', en: 'Secondary literature list', de: 'Liste der Sekundärliteratur' },
          note: { zh: 'Henrich、Allison、邓晓芒', en: 'Henrich, Allison, Deng Xiaomang', de: 'Henrich, Allison, Deng Xiaomang' },
        },
        {
          id: 'k5', kind: 'doc', by: 'su', when: '2026-09-11T21:00',
          title: { zh: '讲稿：范畴的客观有效性', en: 'Talk: the objective validity of the categories', de: 'Vortrag: Die objektive Gültigkeit der Kategorien' },
        },
      ],
      events: [
        { by: 'he', act: 'addWork', ref: { zh: 'B 版演绎论证结构（第三稿）', en: 'Structure of the B Deduction (3rd draft)', de: 'Aufbau der B-Deduktion (3. Fassung)' }, when: '2026-10-16T22:40' },
        { by: 'tang', act: 'addWork', ref: { zh: '第三次讨论纪要', en: 'Notes from the third session', de: 'Protokoll der dritten Sitzung' }, when: '2026-10-02T22:30' },
        { by: 'su', act: 'merge', ref: { zh: '二手文献清单', en: 'Secondary literature list', de: 'Liste der Sekundärliteratur' }, when: '2026-09-26T09:00' },
        { by: 'su', act: 'markActive', when: '2026-09-05T08:00' },
        { by: 'luo', act: 'open', when: '2026-09-04T22:10' },
      ],
      notes: [
        { by: 'tang', when: '2026-10-02T22:50', text: { zh: '我倾向于读成两步。§21 自己说了“证明的开端”，说明还没完。', en: 'I read it as two steps. §21 itself speaks of “the beginning of a deduction”, so it isn’t finished yet.', de: 'Ich lese es als zwei Schritte. §21 spricht selbst vom „Anfang einer Deduktion“, also ist sie noch nicht fertig.' } },
        { by: 'luo', when: '2026-10-09T23:10', text: { zh: '同意两步，但第二步到底多证了什么，我还是说不出来。', en: 'Agreed on two steps, but I still can’t say what the second step adds.', de: 'Einverstanden mit zwei Schritten, aber was der zweite hinzufügt, kann ich immer noch nicht sagen.' } },
        { by: 'he', when: '2026-10-16T22:45', text: { zh: '第三稿里把第二步写成“范围的扩展”：从“被统一的直观”扩到“我们的一切感性直观”。周五请挑错。', en: 'The 3rd draft treats step two as widening the scope: from “unified intuition” to “all our sensible intuition”. Please find holes on Friday.', de: 'Die 3. Fassung liest den zweiten Schritt als Erweiterung: von „vereinigter Anschauung“ zu „aller unserer sinnlichen Anschauung“. Bitte am Freitag Lücken finden.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '“物自身”还是“物自体”', en: 'Ding an sich: 物自身 or 物自体?', de: 'Ding an sich: 物自身 oder 物自体?' },
      summary: {
        zh: '两个译本用词不同，讨论时总在打架。需要定一个，写进术语表。',
        en: 'The two translations disagree and it keeps derailing our sessions. Pick one and put it in the glossary.',
        de: 'Die beiden Übersetzungen weichen ab, das stört jede Sitzung. Eine wählen und ins Glossar schreiben.',
      },
      status: 'active', by: 'xu', opened: '2026-10-01', updated: '2026-10-17T21:05',
      body: [], works: [], workCount: 3, events: [],
    },
    {
      id: '0003',
      title: { zh: '先读完《未来形而上学导论》', en: 'Finish the Prolegomena first', de: 'Erst die Prolegomena zu Ende lesen' },
      summary: {
        zh: '作为进入第一批判的准备。八周读完，每周一人领读。',
        en: 'As a way into the first Critique. Eight weeks, one person leads each week.',
        de: 'Als Einstieg in die erste Kritik. Acht Wochen, jede Woche leitet jemand anderes.',
      },
      status: 'solved', by: 'su', opened: '2026-01-08', updated: '2026-03-05',
      body: [], works: [], workCount: 8, events: [],
    },
    {
      id: '0004',
      title: { zh: '下学期读第二批判还是第三批判', en: 'Next term: the second or the third Critique?', de: 'Nächstes Semester: zweite oder dritte Kritik?' },
      summary: {
        zh: '十二月前决定，好让大家提前买书。',
        en: 'Decide before December so everyone can get the book in time.',
        de: 'Bis Dezember entscheiden, damit alle rechtzeitig das Buch haben.',
      },
      status: 'open', by: 'ge', opened: '2026-10-12', updated: '2026-10-12',
      body: [], works: [], workCount: 0, events: [],
    },
  ],
  recent: [
    { by: 'xu', problem: '0002', act: 'propose', ref: translateDing, when: '2026-10-17T21:05' },
    { by: 'he', problem: '0001', act: 'addWork', ref: { zh: 'B 版演绎论证结构（第三稿）', en: 'Structure of the B Deduction (3rd draft)', de: 'Aufbau der B-Deduktion (3. Fassung)' }, when: '2026-10-16T22:40' },
    { by: 'ge', problem: '0004', act: 'openProblem', ref: { zh: '下学期读第二批判还是第三批判', en: 'Next term: the second or the third Critique?', de: 'Nächstes Semester: zweite oder dritte Kritik?' }, when: '2026-10-12T19:00' },
    { by: 'he', problem: '0002', act: 'update', ref: glossary, when: '2026-10-09T23:00' },
  ],
  proposals: [
    {
      id: 'p1',
      title: translateDing,
      by: 'xu',
      problem: '0002',
      when: '2026-10-17T21:05',
      reason: {
        zh: '我们以李秋零译本为准，它用“物自身”。术语表里两种写法混用，纪要也跟着乱。统一之后，“本体”留给 Noumenon。',
        en: 'We read Li Qiuling’s translation, which uses 物自身. The glossary mixes both, and our notes follow suit. Once unified, 本体 is kept for Noumenon.',
        de: 'Wir lesen Li Qiulings Übersetzung, die 物自身 verwendet. Das Glossar mischt beide Formen, unsere Protokolle auch. Danach bleibt 本体 für Noumenon reserviert.',
      },
      files: [
        {
          title: glossary,
          kind: 'doc',
          lines: [
            { kind: 'same', text: { zh: 'Anschauung — 直观', en: 'Anschauung — 直观 (intuition)', de: 'Anschauung — 直观' } },
            { kind: 'same', text: { zh: 'Erscheinung — 显象', en: 'Erscheinung — 显象 (appearance)', de: 'Erscheinung — 显象' } },
            { kind: 'del', text: { zh: 'Ding an sich — 物自体', en: 'Ding an sich — 物自体 (thing in itself)', de: 'Ding an sich — 物自体' } },
            { kind: 'add', text: { zh: 'Ding an sich — 物自身（李秋零译本用法）', en: 'Ding an sich — 物自身 (thing in itself; Li Qiuling’s usage)', de: 'Ding an sich — 物自身 (Gebrauch bei Li Qiuling)' } },
            { kind: 'del', text: { zh: 'Noumenon — 本体、物自体', en: 'Noumenon — 本体, 物自体', de: 'Noumenon — 本体, 物自体' } },
            { kind: 'add', text: { zh: 'Noumenon — 本体（不再与物自身混用）', en: 'Noumenon — 本体 (no longer used for the thing in itself)', de: 'Noumenon — 本体 (nicht mehr für das Ding an sich)' } },
            { kind: 'same', text: { zh: 'Verstand — 知性', en: 'Verstand — 知性 (understanding)', de: 'Verstand — 知性' } },
          ],
        },
      ],
    },
  ],
  board: [
    {
      id: 'b2', by: 'su', when: '2026-10-17T10:00',
      title: { zh: '本周改到周五晚上', en: 'This week we meet on Friday evening', de: 'Diese Woche treffen wir uns Freitagabend' },
      text: {
        zh: '周四我出差。周五 19:30，地点不变。读 B 版 §24–§27，十二页。',
        en: 'I’m travelling on Thursday. Friday at 19:30, same place. Read B §24–§27, twelve pages.', de: 'Donnerstag bin ich verreist. Freitag um 19:30, gleicher Ort. Lektüre B §24–§27, zwölf Seiten.',
      },
      replies: [
        { by: 'xu', when: '2026-10-17T12:40', text: { zh: '可以。我会把术语表的提案先交上来，周五一起看。', en: 'Fine. I’ll submit my glossary proposal first so we can look at it on Friday.', de: 'Gut. Ich reiche meinen Glossar-Vorschlag vorher ein, dann sehen wir ihn uns am Freitag an.' } },
        { by: 'tang', when: '2026-10-17T18:03', text: { zh: '周五我要晚到半小时。', en: 'I’ll be half an hour late on Friday.', de: 'Am Freitag komme ich eine halbe Stunde später.' } },
      ],
    },
    {
      id: 'b1', by: 'he', when: '2026-09-30T21:00',
      title: { zh: '纪要轮值表', en: 'Note-taking rota', de: 'Protokoll-Reihenfolge' },
      text: {
        zh: '每次读书会由一人写纪要，按座位顺时针轮。纪要当晚交，挂在对应的问题下面。',
        en: 'One person writes the notes each session, clockwise round the table. Notes go in the same night, under the matching problem.', de: 'Jede Sitzung schreibt eine Person das Protokoll, im Uhrzeigersinn um den Tisch. Das Protokoll kommt noch am selben Abend unter das passende Problem.',
      },
      replies: [],
    },
  ],
  decided: [
    { title: { zh: '二手文献清单', en: 'Secondary literature list', de: 'Liste der Sekundärliteratur' }, by: 'ge', result: 'merged', when: '2026-09-26' },
    { title: { zh: '改成每两周读一次', en: 'Meet every other week instead', de: 'Nur noch alle zwei Wochen treffen' }, by: 'luo', result: 'rejected', when: '2026-05-14' },
  ],
}
