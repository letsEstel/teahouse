// 梧桐里三号院：一个居民院子，解决车位、垃圾、楼道灯这类琐事。
import type { House } from './types'

export const wutong: House = {
  id: 'wutong',
  name: { zh: '梧桐里三号院', en: 'Wutong Lane No. 3', de: 'Wutong-Gasse Nr. 3' },
  nameLines: { zh: ['梧桐里', '三号院'], en: ['Wutong Lane', 'No. 3'], de: ['Wutong-Gasse', 'Nr. 3'] },
  sign: {
    zh: '院里四十二户。车位、垃圾、楼道灯，咱们自己管。',
    en: 'Forty-two households. Parking, bins, stairwell lights — we look after it ourselves.',
    de: 'Zweiundvierzig Haushalte. Parkplätze, Mülltonnen, Treppenlicht – darum kümmern wir uns selbst.',
  },
  founded: '2026-03-14',
  capacity: 15,
  me: 'chen',
  members: [
    { id: 'chen', name: { zh: '陈立', en: 'Chen Li', de: 'Chen Li' }, role: 'host', joined: '2026-03-14' },
    { id: 'zhou', name: { zh: '周小满', en: 'Zhou Xiaoman', de: 'Zhou Xiaoman' }, role: 'steward', joined: '2026-03-14' },
    { id: 'wang', name: { zh: '王阿姨', en: 'Auntie Wang', de: 'Tante Wang' }, role: 'guest', joined: '2026-03-16' },
    { id: 'li', name: { zh: '李建国', en: 'Li Jianguo', de: 'Li Jianguo' }, role: 'guest', joined: '2026-03-16' },
    { id: 'zhao', name: { zh: '赵一鸣', en: 'Zhao Yiming', de: 'Zhao Yiming' }, role: 'guest', joined: '2026-03-22' },
    { id: 'sun', name: { zh: '孙婷', en: 'Sun Ting', de: 'Sun Ting' }, role: 'guest', joined: '2026-04-02' },
    { id: 'wu', name: { zh: '吴师傅', en: 'Master Wu', de: 'Meister Wu' }, role: 'guest', joined: '2026-04-09' },
    { id: 'zheng', name: { zh: '郑可', en: 'Zheng Ke', de: 'Zheng Ke' }, role: 'guest', joined: '2026-05-30' },
    { id: 'lin', name: { zh: '林晚', en: 'Lin Wan', de: 'Lin Wan' }, role: 'guest', joined: '2026-09-21' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '小区停车位不好找', en: 'Hard to find a parking space', de: 'Kaum freie Parkplätze im Hof' },
      summary: {
        zh: '晚上七点以后院内车位全满，外来车占位，住户只能停到两条街外。',
        en: 'After 7 pm every space is full, outside cars take spots, and residents park two streets away.',
        de: 'Ab 19 Uhr ist alles voll, Fremdfahrzeuge blockieren Plätze, Anwohner parken zwei Straßen weiter.',
      },
      status: 'active',
      by: 'wang',
      opened: '2026-03-16',
      updated: '2026-10-18T18:02',
      body: [
        {
          heading: { zh: '现状', en: 'Now', de: 'Ist-Zustand' },
          text: {
            zh: '院内共 42 个车位，登记住户车辆 57 辆。晚高峰（19:00 以后）车位全满，其中约 8 个被外来车辆长期占用。',
            en: '42 spaces, 57 registered resident cars. From 19:00 every space is taken, about 8 of them long-term by outside cars.',
            de: '42 Stellplätze, 57 gemeldete Anwohnerautos. Ab 19:00 ist alles belegt, etwa 8 Plätze dauerhaft von Fremdfahrzeugen.',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal', de: 'Ziel' },
          text: {
            zh: '年底前让每户至少有一个可预期的停车位置；外来车辆占位降到 2 个以下。',
            en: 'By year end, every household can count on a place to park; outside cars take fewer than 2 spaces.',
            de: 'Bis Jahresende hat jeder Haushalt einen verlässlichen Stellplatz; Fremdfahrzeuge belegen weniger als 2 Plätze.',
          },
        },
        {
          heading: { zh: '约束', en: 'Constraints', de: 'Rahmenbedingungen' },
          text: {
            zh: '不新增地面硬化；物业不额外收费；方案需要业委会表决通过。',
            en: 'No new paving; no extra fees from management; the owners’ committee must vote it through.',
            de: 'Keine neue Flächenversiegelung; keine Zusatzgebühren der Hausverwaltung; der Eigentümerbeirat muss zustimmen.',
          },
        },
      ],
      works: [
        {
          id: 'w1', kind: 'image', by: 'zhou', when: '2026-10-18T18:02', preview: { chart: 'parking' },
          title: { zh: '车位分布图（实测）', en: 'Parking map (measured)', de: 'Parkplatzplan (vermessen)' },
          note: { zh: '42 个车位逐一标号', en: 'All 42 spaces numbered', de: 'Alle 42 Plätze nummeriert' },
        },
        {
          id: 'w2', kind: 'plan', by: 'zhao', when: '2026-10-17T21:40',
          title: { zh: '错峰共享方案 v2', en: 'Time-sharing plan v2', de: 'Zeitversetzte Platzteilung v2' },
          note: { zh: '白天空位开放给夜班住户', en: 'Daytime spaces open to night-shift residents', de: 'Tagsüber freie Plätze für Nachtschicht-Anwohner' },
          lines: [
            { zh: '一、适用范围', en: '1. Scope', de: '1. Geltungsbereich' },
            { zh: '院内全部 42 个地面车位。', en: 'All 42 surface spaces in the courtyard.', de: 'Alle 42 Stellplätze im Hof.' },
            { zh: '二、时段划分', en: '2. Time slots', de: '2. Zeitfenster' },
            { zh: '07:00–19:00 为日间时段：登记的夜班住户可停任意空位。', en: '07:00–19:00 day slot: registered night-shift residents may use any free space.', de: '07:00–19:00 Tagesfenster: gemeldete Nachtschicht-Anwohner dürfen jeden freien Platz nutzen.' },
            { zh: '19:00–07:00 为夜间时段：车位优先归属固定住户。', en: '19:00–07:00 night slot: spaces go to regular residents first.', de: '19:00–07:00 Nachtfenster: Plätze gehören zuerst den Stammanwohnern.' },
          ],
        },
        {
          id: 'w3', kind: 'table', by: 'li', when: '2026-10-12T10:00',
          title: { zh: '地锁采购报价', en: 'Parking lock quotes', de: 'Angebote für Parkbügel' },
          note: { zh: '三家对比，含安装', en: 'Three suppliers, installation included', de: 'Drei Anbieter, inklusive Montage' },
        },
        {
          id: 'w4', kind: 'doc', by: 'wang', when: '2026-10-09T21:30',
          title: { zh: '一周占位记录', en: 'One week of occupancy notes', de: 'Belegungsprotokoll einer Woche' },
          note: { zh: '每晚 21:00 拍照统计', en: 'Photographed every night at 21:00', de: 'Jeden Abend um 21:00 fotografiert' },
        },
        { id: 'w10', kind: 'link', by: 'sun', when: '2026-10-18T19:30', url: 'https://docs.qq.com/sheet/DUEtPaXxYzAbCdEf', title: { zh: '车位意见收集表', en: 'Parking feedback sheet', de: 'Rückmeldebogen Parkplätze' }, note: { zh: '大家在腾讯文档里一起填，周日汇总后贴回茶馆', en: 'Everyone fills it in together on Tencent Docs; summed up here on Sunday', de: 'Alle füllen es gemeinsam in Tencent Docs aus; sonntags hier zusammengefasst' } },
        { id: 'w5', kind: 'doc', by: 'chen', when: '2026-10-03T15:00', title: { zh: '致物业的函（草稿）', en: 'Letter to the management (draft)', de: 'Schreiben an die Hausverwaltung (Entwurf)' } },
        { id: 'w6', kind: 'table', by: 'sun', when: '2026-09-27T12:00', title: { zh: '周边收费停车场价目', en: 'Nearby car park prices', de: 'Preise der Parkhäuser in der Nähe' } },
        { id: 'w7', kind: 'image', by: 'wu', when: '2026-09-20T09:00', title: { zh: '外来车辆照片汇总', en: 'Photos of outside cars', de: 'Fotos der Fremdfahrzeuge' } },
        { id: 'w8', kind: 'plan', by: 'zhao', when: '2026-09-18T20:00', title: { zh: '错峰共享方案 v1', en: 'Time-sharing plan v1', de: 'Zeitversetzte Platzteilung v1' } },
        { id: 'w9', kind: 'doc', by: 'zheng', when: '2026-09-11T19:00', title: { zh: '住户问卷结果', en: 'Resident survey results', de: 'Ergebnisse der Anwohnerumfrage' } },
      ],
      events: [
        { by: 'zhou', act: 'addWork', ref: { zh: '车位分布图（实测）', en: 'Parking map (measured)', de: 'Parkplatzplan (vermessen)' }, when: '2026-10-18T18:02' },
        { by: 'zhao', act: 'propose', ref: { zh: '把夜间临时停车改为错峰共享', en: 'Replace night-time parking with time-sharing', de: 'Nachtparken durch zeitversetzte Teilung ersetzen' }, when: '2026-10-17T22:14' },
        { by: 'chen', act: 'merge', ref: { zh: '地锁采购报价', en: 'Parking lock quotes', de: 'Angebote für Parkbügel' }, when: '2026-10-12T10:20' },
        { by: 'wang', act: 'update', ref: { zh: '一周占位记录', en: 'One week of occupancy notes', de: 'Belegungsprotokoll einer Woche' }, when: '2026-10-09T21:30' },
        { by: 'chen', act: 'markActive', when: '2026-09-18T20:30' },
        { by: 'wang', act: 'open', when: '2026-03-16T09:00' },
      ],
      notes: [
        { by: 'li', when: '2026-10-13T08:40', text: { zh: '地锁报价如果用不上，报价表能不能留着？别的院子可能用得上。', en: 'If we don’t need the lock quotes, can we keep the quote table anyway? Another courtyard might use it.', de: 'Wenn wir die Bügel-Angebote nicht brauchen, können wir die Angebotstabelle trotzdem behalten? Ein anderer Hof kann sie vielleicht gebrauchen.' } },
        { by: 'zhao', when: '2026-10-17T22:30', text: { zh: 'v2 里夜班住户要提前登记。登记表谁来收？', en: 'In v2 night-shift residents register in advance. Who collects the forms?', de: 'In v2 melden sich Nachtschicht-Anwohner vorher an. Wer sammelt die Formulare ein?' } },
        { by: 'zhou', when: '2026-10-18T09:10', text: { zh: '我来收。门卫室放一个本子，每周日拍照传到这里。', en: 'I will. There’ll be a notebook in the gatehouse; I’ll photograph it every Sunday and post it here.', de: 'Mache ich. Im Pförtnerhaus liegt ein Heft; ich fotografiere es jeden Sonntag und stelle es hier ein.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '垃圾分类点的气味', en: 'The smell at the recycling point', de: 'Geruch an der Müllsammelstelle' },
      summary: {
        zh: '夏季分类点紧挨 2 号楼窗下，异味严重，需要挪位置或加清运频次。',
        en: 'In summer the bins sit right under Building 2’s windows. Move them or collect more often.',
        de: 'Im Sommer stehen die Tonnen direkt unter den Fenstern von Haus 2. Versetzen oder öfter abholen.',
      },
      status: 'active', by: 'sun', opened: '2026-06-02', updated: '2026-10-17T10:00',
      body: [], works: [], workCount: 5, events: [],
    },
    {
      id: '0003',
      title: { zh: '楼道灯坏了三个月', en: 'Stairwell lights out for three months', de: 'Treppenlicht seit drei Monaten kaputt' },
      summary: {
        zh: '3 号楼 4–6 层声控灯失灵。已找到电工，费用由维修基金支出。',
        en: 'Sound-activated lights on floors 4–6 of Building 3. Electrician found, paid from the repair fund.',
        de: 'Geräuschgesteuertes Licht in Haus 3, Stock 4–6. Elektriker gefunden, bezahlt aus der Instandhaltungsrücklage.',
      },
      status: 'solved', by: 'wu', opened: '2026-04-10', updated: '2026-07-21',
      body: [], works: [], workCount: 4, events: [],
    },
    {
      id: '0004',
      title: { zh: '给院里独居老人订午餐', en: 'Lunch for neighbours who live alone', de: 'Mittagessen für alleinlebende Nachbarn' },
      summary: {
        zh: '院内 6 位独居老人，想联系附近食堂按周订餐、轮流送到门口。',
        en: 'Six older neighbours live alone. Order weekly from a nearby canteen and take turns delivering.',
        de: 'Sechs ältere Nachbarn leben allein. Wöchentlich bei einer Kantine bestellen und abwechselnd liefern.',
      },
      status: 'open', by: 'lin', opened: '2026-10-15', updated: '2026-10-15',
      body: [], works: [], workCount: 2, events: [],
    },
    {
      id: '0005',
      title: { zh: '年底公共维修基金怎么花', en: 'How to spend the repair fund by year end', de: 'Wofür die Instandhaltungsrücklage bis Jahresende?' },
      summary: {
        zh: '账上余 3.2 万元，需要在 12 月业委会前列出优先级清单。',
        en: '¥32,000 left. We need a priority list before the December committee meeting.',
        de: '32.000 ¥ übrig. Vor der Beiratssitzung im Dezember brauchen wir eine Prioritätenliste.',
      },
      status: 'open', by: 'chen', opened: '2026-10-08', updated: '2026-10-14',
      body: [], works: [], workCount: 3, events: [],
    },
  ],
  recent: [
    { by: 'zhou', problem: '0001', act: 'addWork', ref: { zh: '车位分布图（实测）', en: 'Parking map (measured)', de: 'Parkplatzplan (vermessen)' }, when: '2026-10-18T18:02' },
    { by: 'zhao', problem: '0001', act: 'propose', ref: { zh: '把夜间临时停车改为错峰共享', en: 'Replace night-time parking with time-sharing', de: 'Nachtparken durch zeitversetzte Teilung ersetzen' }, when: '2026-10-17T22:14' },
    { by: 'sun', problem: '0002', act: 'addWork', ref: { zh: '清运时间表', en: 'Collection schedule', de: 'Abholplan' }, when: '2026-10-17T10:00' },
    { by: 'lin', problem: '0004', act: 'openProblem', ref: { zh: '给院里独居老人订午餐', en: 'Lunch for neighbours who live alone', de: 'Mittagessen für alleinlebende Nachbarn' }, when: '2026-10-15T08:30' },
    { by: 'chen', problem: '0005', act: 'addWork', ref: { zh: '维修基金余额明细', en: 'Repair fund balance', de: 'Kontostand der Rücklage' }, when: '2026-10-14T19:00' },
  ],
  proposals: [
    {
      id: 'p1',
      title: { zh: '把夜间临时停车改为错峰共享', en: 'Replace night-time parking with time-sharing', de: 'Nachtparken durch zeitversetzte Teilung ersetzen' },
      by: 'zhao',
      problem: '0001',
      when: '2026-10-17T22:14',
      reason: {
        zh: 'v1 只解决了夜间外来车，没有考虑夜班住户。问卷里有 11 户上夜班，白天车位空着、晚上回来没地方。改成按时段共享，不需要装地锁。',
        en: 'v1 only dealt with outside cars at night and forgot night-shift residents. Eleven households in the survey work nights: their spaces sit empty by day and there’s nowhere to park when they return. Sharing by time slot needs no parking locks.',
        de: 'v1 hat nur die Fremdfahrzeuge nachts gelöst und die Nachtschicht vergessen. Elf Haushalte arbeiten laut Umfrage nachts: tagsüber stehen ihre Plätze leer, abends finden sie keinen. Teilung nach Zeitfenstern braucht keine Parkbügel.',
      },
      files: [
        {
          title: { zh: '错峰共享方案', en: 'Time-sharing plan', de: 'Zeitversetzte Platzteilung' },
          kind: 'plan',
          lines: [
            { kind: 'same', text: { zh: '一、适用范围', en: '1. Scope', de: '1. Geltungsbereich' } },
            { kind: 'same', text: { zh: '院内全部 42 个地面车位。', en: 'All 42 surface spaces in the courtyard.', de: 'Alle 42 Stellplätze im Hof.' } },
            { kind: 'same', text: { zh: '', en: '', de: '' } },
            { kind: 'same', text: { zh: '二、时段划分', en: '2. Time slots', de: '2. Zeitfenster' } },
            { kind: 'del', text: { zh: '19:00–07:00 禁止外来车辆进入，由门卫登记放行。', en: '19:00–07:00 no outside cars; the gatekeeper registers exceptions.', de: '19:00–07:00 keine Fremdfahrzeuge; der Pförtner trägt Ausnahmen ein.' } },
            { kind: 'add', text: { zh: '07:00–19:00 为日间时段：登记的夜班住户可停任意空位。', en: '07:00–19:00 day slot: registered night-shift residents may use any free space.', de: '07:00–19:00 Tagesfenster: gemeldete Nachtschicht-Anwohner dürfen jeden freien Platz nutzen.' } },
            { kind: 'add', text: { zh: '19:00–07:00 为夜间时段：车位优先归属固定住户，夜班住户须在 19:30 前驶离。', en: '19:00–07:00 night slot: spaces go to regular residents first; night-shift cars leave by 19:30.', de: '19:00–07:00 Nachtfenster: Plätze gehören zuerst den Stammanwohnern; Nachtschicht-Autos fahren bis 19:30 weg.' } },
            { kind: 'same', text: { zh: '', en: '', de: '' } },
            { kind: 'same', text: { zh: '三、执行方式', en: '3. How it works', de: '3. Umsetzung' } },
            { kind: 'del', text: { zh: '在 8 个高频被占车位安装地锁，钥匙由住户自管。', en: 'Fit parking locks on the 8 most-taken spaces; residents keep the keys.', de: 'Parkbügel an den 8 meistbelegten Plätzen; die Schlüssel verwalten die Anwohner.' } },
            { kind: 'add', text: { zh: '不安装地锁。每个车位喷涂编号，在院门口张贴当日共享表。', en: 'No parking locks. Every space gets a painted number; the day’s sharing table hangs at the gate.', de: 'Keine Parkbügel. Jeder Platz bekommt eine Nummer, der Tagesplan hängt am Hoftor.' } },
            { kind: 'add', text: { zh: '违规三次的车辆，由伙计统一联系车主。', en: 'After three violations, the steward contacts the car’s owner.', de: 'Nach drei Verstößen kontaktiert der Gehilfe die Halter.' } },
            { kind: 'same', text: { zh: '外来车辆仍需门卫登记。', en: 'Outside cars still register with the gatekeeper.', de: 'Fremdfahrzeuge melden sich weiterhin beim Pförtner an.' } },
          ],
        },
        {
          title: { zh: '地锁采购报价', en: 'Parking lock quotes', de: 'Angebote für Parkbügel' },
          kind: 'table',
          lines: [
            { kind: 'same', text: { zh: '供应商　　　单价　　数量　　小计', en: 'Supplier      Unit   Qty   Subtotal', de: 'Anbieter      Preis  Anz.  Summe' } },
            { kind: 'del', text: { zh: '安途五金　　¥186　　8　　　¥1,488', en: 'Antu Hardware   ¥186   8   ¥1,488', de: 'Antu Eisenwaren   186 ¥   8   1.488 ¥' } },
            { kind: 'add', text: { zh: '（方案 v2 不再采购地锁，本表作废留档）', en: '(Plan v2 buys no locks; this table is void and kept for the record)', de: '(Plan v2 kauft keine Bügel; diese Tabelle ist ungültig und bleibt archiviert)' } },
          ],
        },
      ],
    },
    {
      id: 'p2',
      title: { zh: '补充 2 号楼窗下实测气味记录', en: 'Add measured smell readings below Building 2', de: 'Geruchsmessungen unter Haus 2 ergänzen' },
      by: 'sun',
      problem: '0002',
      when: '2026-10-16T09:00',
      files: [],
    },
  ],
  board: [
    {
      id: 'b2', by: 'chen', when: '2026-10-16T20:00',
      title: { zh: '10 月 25 日（周六）上午院内大扫除', en: 'Courtyard clean-up, Saturday 25 October, morning', de: 'Hofputz am Samstag, 25. Oktober, vormittags' },
      text: {
        zh: '九点在 2 号楼前集合，工具由物业提供。顺便给车位喷编号，需要四个人帮忙。能来的在下面写一句。',
        en: 'Meet at 9 in front of Building 2; the management provides the tools. We’ll also paint the parking numbers and need four helpers. Write a line below if you can come.', de: 'Treffpunkt um 9 vor Haus 2; das Werkzeug stellt die Hausverwaltung. Dabei malen wir auch die Parkplatznummern und brauchen vier Helfer. Wer kommen kann, schreibt unten eine Zeile.',
      },
      replies: [
        { by: 'wang', when: '2026-10-16T20:31', text: { zh: '我来，带两把扫帚。', en: 'I’ll come and bring two brooms.', de: 'Ich komme und bringe zwei Besen mit.' } },
        { by: 'wu', when: '2026-10-17T07:50', text: { zh: '喷漆的活我熟，算我一个。', en: 'I know how to spray-paint. Count me in.', de: 'Mit Sprühfarbe kenne ich mich aus. Ich bin dabei.' } },
        { by: 'lin', when: '2026-10-17T12:02', text: { zh: '上午要陪老人看病，下午可以来收尾。', en: 'I’m taking a neighbour to the doctor in the morning, but I can help finish in the afternoon.', de: 'Vormittags begleite ich eine Nachbarin zum Arzt, nachmittags helfe ich beim Abschluss.' } },
      ],
    },
    {
      id: 'b1', by: 'zhou', when: '2026-10-08T19:00',
      title: { zh: '停车方案 11 月 6 日在业委会表决', en: 'Parking plan goes to the committee vote on 6 November', de: 'Parkplan kommt am 6. November im Beirat zur Abstimmung' },
      text: {
        zh: '表决前一周定稿。有意见请写在“小区停车位不好找”下面，别只在群里说，群里的话会被冲掉。',
        en: 'Final draft a week before the vote. Please write comments under “Hard to find a parking space”, not just in the group chat, where they get lost.', de: 'Endfassung eine Woche vor der Abstimmung. Anmerkungen bitte unter „Kaum freie Parkplätze im Hof“ schreiben, nicht nur in den Gruppenchat, dort gehen sie unter.',
      },
      replies: [{ by: 'zhao', when: '2026-10-08T21:15', text: { zh: '收到，v2 周末前提交。', en: 'Got it. v2 will be in before the weekend.', de: 'Verstanden. v2 kommt vor dem Wochenende.' } }],
    },
  ],
  decided: [
    { title: { zh: '地锁采购报价', en: 'Parking lock quotes', de: 'Angebote für Parkbügel' }, by: 'li', result: 'merged', when: '2026-10-12' },
    { title: { zh: '楼道灯维修费用分摊', en: 'Splitting the stairwell light repair', de: 'Kostenaufteilung Treppenlicht' }, by: 'wu', result: 'merged', when: '2026-07-20' },
    { title: { zh: '停车位按楼栋抽签', en: 'Parking lottery by building', de: 'Parkplatzlos nach Hausnummer' }, by: 'zheng', result: 'rejected', when: '2026-06-03' },
  ],
}
