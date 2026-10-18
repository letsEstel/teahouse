// 旧鞋带乐队：一支业余乐队筹备第一场演出。当前用户是老板（主唱鹿鸣）。
import type { House } from './types'

const venues = { zh: '三处场地对比', en: 'Three venues compared', de: 'Drei Orte im Vergleich' }
const setlist = { zh: '曲目单', en: 'Setlist', de: 'Setlist' }
const swapSong = { zh: '曲目单：用新歌《第七站》换掉《夏天的雨》', en: 'Setlist: swap “Summer Rain” for the new song “Stop Seven”', de: 'Setlist: „Sommerregen“ durch den neuen Song „Station Sieben“ ersetzen' }

export const band: House = {
  id: 'band',
  name: { zh: '旧鞋带乐队', en: 'The Old Laces', de: 'The Old Laces' },
  nameLines: { zh: ['旧鞋带', '乐队'], en: ['The Old', 'Laces'], de: ['The Old', 'Laces'] },
  sign: {
    zh: '十二月，第一场演出。四十张票，不亏钱就算赢。',
    en: 'December, our first gig. Forty tickets. Breaking even counts as a win.',
    de: 'Dezember, unser erster Auftritt. Vierzig Karten. Ohne Verlust ist schon gewonnen.',
  },
  founded: '2026-09-12',
  capacity: 15,
  me: 'lu',
  members: [
    { id: 'lu', name: { zh: '鹿鸣', en: 'Lu Ming', de: 'Lu Ming' }, role: 'boss', joined: '2026-09-12' },
    { id: 'fang', name: { zh: '方圆', en: 'Fang Yuan', de: 'Fang Yuan' }, role: 'heir', joined: '2026-09-12' },
    { id: 'zhu', name: { zh: '小朱', en: 'Xiao Zhu', de: 'Xiao Zhu' }, role: 'guest', joined: '2026-09-12' },
    { id: 'tao', name: { zh: '陶然', en: 'Tao Ran', de: 'Tao Ran' }, role: 'guest', joined: '2026-09-13' },
    { id: 'susu', name: { zh: '苏苏', en: 'Susu', de: 'Susu' }, role: 'guest', joined: '2026-09-20' },
    { id: 'ji', name: { zh: '老季', en: 'Old Ji', de: 'Der alte Ji' }, role: 'guest', joined: '2026-10-01' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '演出场地定哪里', en: 'Where do we play?', de: 'Wo spielen wir?' },
      summary: {
        zh: '看了三处地方，各有各的毛病。要能现场打鼓，还要能卖出四十张票。',
        en: 'We’ve seen three places, each with a catch. We need live drums and room for forty tickets.',
        de: 'Drei Orte angesehen, jeder mit einem Haken. Wir brauchen echtes Schlagzeug und Platz für vierzig Karten.',
      },
      status: 'active',
      by: 'fang',
      opened: '2026-09-15',
      updated: '2026-10-18T22:40',
      body: [
        {
          heading: { zh: '现状', en: 'Now', de: 'Stand' },
          text: {
            zh: '看了三处：学校旁的小 livehouse（80 人，场租 1500 元）、咖啡馆二楼（40 人，不收场租但有最低消费）、朋友的仓库（地方大，没有音响）。',
            en: 'Three options: the small live house by the school (80 people, ¥1,500 rent), upstairs at a café (40 people, no rent but a minimum spend), a friend’s warehouse (lots of room, no PA).',
            de: 'Drei Optionen: der kleine Club an der Schule (80 Leute, 1.500 ¥ Miete), der erste Stock eines Cafés (40 Leute, keine Miete, aber Mindestverzehr), das Lager eines Freundes (viel Platz, keine Anlage).',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal', de: 'Ziel' },
          text: {
            zh: '12 月 13 日或 20 日，周六晚上，一个能卖出四十张票的地方。',
            en: 'Saturday 13 or 20 December, in the evening, somewhere we can sell forty tickets.',
            de: 'Samstag, 13. oder 20. Dezember, abends, ein Ort für vierzig verkaufte Karten.',
          },
        },
        {
          heading: { zh: '约束', en: 'Limits', de: 'Rahmen' },
          text: {
            zh: '总预算三千元；鼓必须能真打，只能用电鼓的地方不考虑。',
            en: '¥3,000 in total; the drums have to be real, so electronic-kit-only places are out.',
            de: 'Insgesamt 3.000 ¥; das Schlagzeug muss echt sein, Orte nur mit E-Drums fallen weg.',
          },
        },
      ],
      works: [
        {
          id: 'm1', kind: 'plan', by: 'fang', when: '2026-10-18T22:40', title: venues,
          note: { zh: '容量、费用、能不能打鼓', en: 'Capacity, cost, live drums or not', de: 'Kapazität, Kosten, echtes Schlagzeug ja/nein' },
          preview: {
            excerpt: {
              zh: 'livehouse：80 人，场租 1500，音响齐全，鼓可以真打。\n咖啡馆：40 人，不收场租，最低消费 1200，只能用电鼓。\n仓库：人数不限，不收钱，没有音响，要老季借一套。',
              en: 'Live house: 80 people, ¥1,500 rent, full PA, real drums allowed.\nCafé: 40 people, no rent, ¥1,200 minimum spend, electronic drums only.\nWarehouse: any number, free, no PA — Old Ji would have to borrow one.',
              de: 'Club: 80 Leute, 1.500 ¥ Miete, volle Anlage, echtes Schlagzeug erlaubt.\nCafé: 40 Leute, keine Miete, 1.200 ¥ Mindestverzehr, nur E-Drums.\nLager: beliebig viele, kostenlos, keine Anlage — der alte Ji müsste eine leihen.',
            },
          },
        },
        {
          id: 'm2', kind: 'bill', by: 'fang', when: '2026-10-16T20:00',
          title: { zh: '演出预算', en: 'Gig budget', de: 'Budget für den Auftritt' },
          note: { zh: '按卖出 40 张票算', en: 'Assuming 40 tickets sold', de: 'Gerechnet mit 40 verkauften Karten' },
        },
        {
          id: 'm3', kind: 'doc', by: 'lu', when: '2026-10-12T23:10', title: setlist,
          note: { zh: '八首，约 40 分钟', en: 'Eight songs, about 40 minutes', de: 'Acht Songs, etwa 40 Minuten' },
          lines: [
            { zh: '1. 旧鞋带', en: '1. Old Laces', de: '1. Old Laces' },
            { zh: '2. 夏天的雨', en: '2. Summer Rain', de: '2. Sommerregen' },
            { zh: '3. 十三路末班车', en: '3. Last Bus on Route 13', de: '3. Letzter Bus der Linie 13' },
            { zh: '4. 天台', en: '4. Rooftop', de: '4. Dachterrasse' },
            { zh: '5. 慢一点（翻唱）', en: '5. Slow Down (cover)', de: '5. Langsamer (Cover)' },
          ],
        },
        {
          id: 'm4', kind: 'image', by: 'susu', when: '2026-10-10T18:30',
          title: { zh: '海报草图（三版）', en: 'Poster sketches (three versions)', de: 'Plakatentwürfe (drei Versionen)' },
        },
      ],
      events: [
        { by: 'fang', act: 'addWork', ref: venues, when: '2026-10-18T22:40' },
        { by: 'tao', act: 'propose', ref: swapSong, when: '2026-10-17T01:20' },
        { by: 'lu', act: 'addWork', ref: setlist, when: '2026-10-12T23:10' },
        { by: 'lu', act: 'markActive', when: '2026-09-16T10:00' },
        { by: 'fang', act: 'open', when: '2026-09-15T21:00' },
      ],
      notes: [
        { by: 'zhu', when: '2026-10-18T22:55', text: { zh: '咖啡馆只能用电鼓，我不太愿意。', en: 'The café only allows electronic drums. I’d rather not.', de: 'Im Café gehen nur E-Drums. Das möchte ich eher nicht.' } },
        { by: 'fang', when: '2026-10-18T23:02', text: { zh: '那就 livehouse 或者仓库。老季，仓库那边的音响你能借到吗？', en: 'Then it’s the live house or the warehouse. Old Ji, can you borrow a PA for the warehouse?', de: 'Dann Club oder Lager. Alter Ji, kannst du für das Lager eine Anlage leihen?' } },
        { by: 'ji', when: '2026-10-18T23:20', text: { zh: '能借一套小的，四十个人够用。但搬的时候要两个人帮忙。', en: 'I can borrow a small one, enough for forty people. But I need two people to help carry it.', de: 'Eine kleine kann ich leihen, reicht für vierzig Leute. Aber beim Tragen brauche ich zwei Helfer.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '海报谁来做', en: 'Who makes the poster?', de: 'Wer macht das Plakat?' },
      summary: {
        zh: '苏苏答应画，但要先定场地和日期才能出终稿。',
        en: 'Susu has agreed to draw it, but needs the venue and date before the final version.',
        de: 'Susu zeichnet es, braucht aber Ort und Datum für die Endfassung.',
      },
      status: 'active', by: 'lu', opened: '2026-09-20', updated: '2026-10-10',
      body: [], works: [], workCount: 3, events: [],
    },
    {
      id: '0003',
      title: { zh: '曲目定八首还是十首', en: 'Eight songs or ten?', de: 'Acht Songs oder zehn?' },
      summary: {
        zh: '十首会更完整，但新歌《第七站》还没排熟。',
        en: 'Ten feels complete, but the new song “Stop Seven” isn’t tight yet.',
        de: 'Zehn wirken runder, aber der neue Song „Station Sieben“ sitzt noch nicht.',
      },
      status: 'open', by: 'tao', opened: '2026-10-11', updated: '2026-10-17',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0004',
      title: { zh: '每周固定排练时间', en: 'A fixed weekly rehearsal time', de: 'Eine feste wöchentliche Probe' },
      summary: {
        zh: '定在周三晚上七点到十点，地方是方圆家楼下的琴房。',
        en: 'Wednesdays 7 to 10 pm, in the practice room below Fang Yuan’s flat.',
        de: 'Mittwochs 19 bis 22 Uhr, im Proberaum unter Fang Yuans Wohnung.',
      },
      status: 'solved', by: 'zhu', opened: '2026-09-13', updated: '2026-09-18',
      body: [], works: [], workCount: 1, events: [],
    },
  ],
  recent: [
    { by: 'fang', problem: '0001', act: 'addWork', ref: venues, when: '2026-10-18T22:40' },
    { by: 'tao', problem: '0003', act: 'propose', ref: swapSong, when: '2026-10-17T01:20' },
    { by: 'lu', problem: '0001', act: 'addWork', ref: setlist, when: '2026-10-12T23:10' },
    { by: 'susu', problem: '0002', act: 'addWork', ref: { zh: '海报草图（三版）', en: 'Poster sketches (three versions)', de: 'Plakatentwürfe (drei Versionen)' }, when: '2026-10-10T18:30' },
  ],
  proposals: [
    {
      id: 'p1',
      title: swapSong,
      by: 'tao',
      problem: '0003',
      when: '2026-10-17T01:20',
      reason: {
        zh: '《夏天的雨》和《天台》太像了，连着听会腻。《第七站》上周排了两遍，副歌已经稳了。',
        en: '“Summer Rain” and “Rooftop” sound too alike back to back. We ran “Stop Seven” twice last week and the chorus is solid now.',
        de: '„Sommerregen“ und „Dachterrasse“ klingen hintereinander zu ähnlich. „Station Sieben“ haben wir letzte Woche zweimal gespielt, der Refrain sitzt jetzt.',
      },
      files: [
        {
          title: setlist,
          kind: 'doc',
          lines: [
            { kind: 'same', text: { zh: '1. 旧鞋带', en: '1. Old Laces', de: '1. Old Laces' } },
            { kind: 'del', text: { zh: '2. 夏天的雨', en: '2. Summer Rain', de: '2. Sommerregen' } },
            { kind: 'add', text: { zh: '2. 第七站（新歌）', en: '2. Stop Seven (new)', de: '2. Station Sieben (neu)' } },
            { kind: 'same', text: { zh: '3. 十三路末班车', en: '3. Last Bus on Route 13', de: '3. Letzter Bus der Linie 13' } },
            { kind: 'same', text: { zh: '4. 天台', en: '4. Rooftop', de: '4. Dachterrasse' } },
            { kind: 'same', text: { zh: '5. 慢一点（翻唱）', en: '5. Slow Down (cover)', de: '5. Langsamer (Cover)' } },
          ],
        },
      ],
    },
  ],
  decided: [
    { title: { zh: '排练定在周三晚上', en: 'Rehearsals on Wednesday evenings', de: 'Proben mittwochabends' }, by: 'zhu', result: 'merged', when: '2026-09-18' },
  ],
  board: [
    {
      id: 'b1', by: 'lu', when: '2026-10-15T12:00',
      title: { zh: '本周三排练：从头到尾走一遍', en: 'Wednesday rehearsal: a full run-through', de: 'Probe am Mittwoch: einmal komplett durch' },
      text: {
        zh: '按曲目单顺序，中间不停。苏苏会来拍几张照片做海报。老季如果有空，帮忙听一下音量。',
        en: 'In setlist order, no stopping. Susu will come and take a few photos for the poster. Old Ji, if you’re free, please listen for the levels.',
        de: 'In der Reihenfolge der Setlist, ohne Pause. Susu macht ein paar Fotos fürs Plakat. Alter Ji, wenn du Zeit hast, hör bitte auf die Lautstärke.',
      },
      replies: [
        { by: 'susu', when: '2026-10-15T13:10', text: { zh: '八点到，带相机。', en: 'I’ll be there at eight with my camera.', de: 'Ich bin um acht da, mit Kamera.' } },
        { by: 'ji', when: '2026-10-15T19:45', text: { zh: '能来，九点前到。', en: 'I can come, before nine.', de: 'Ich komme, vor neun.' } },
      ],
    },
  ],
}
