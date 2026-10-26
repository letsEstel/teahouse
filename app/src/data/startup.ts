// 三人创业团队：三个人给小餐馆做记账工具，外加一位 AI 茶客帮忙整理和起草。当前用户是伙计。
import type { House } from './types'

const pricing = { zh: '定价表', en: 'Price list', de: 'Preisliste' }
const offlinePlan = { zh: '离线方案：第一版只做离线记账', en: 'Offline plan: version one records offline only' }
const cutPrice = { zh: '基础版降到每月 49 元', en: 'Cut the basic plan to ¥49 a month', de: 'Basistarif auf 49 ¥ im Monat senken' }

export const startup: House = {
  id: 'startup',
  name: { zh: '三人创业团队', en: 'Three-Person Startup', de: 'Dreier-Gründerteam' },
  nameLines: { zh: ['三人', '创业团队'], en: ['Three-Person', 'Startup'], de: ['Dreier-', 'Gründerteam'] },
  sign: {
    zh: '让小餐馆不再手写账本。年底前，十个付费用户。',
    en: 'No more handwritten ledgers in small restaurants. Ten paying customers by year end.',
    de: 'Keine handgeschriebenen Kassenbücher mehr in kleinen Restaurants. Zehn zahlende Kunden bis Jahresende.',
  },
  founded: '2026-08-01',
  capacity: 15,
  me: 'song',
  members: [
    { id: 'linz', name: { zh: '林舟', en: 'Lin Zhou', de: 'Lin Zhou' }, role: 'host', joined: '2026-08-01' },
    { id: 'song', name: { zh: '宋雨桐', en: 'Song Yutong', de: 'Song Yutong' }, role: 'steward', joined: '2026-08-01' },
    { id: 'ma', name: { zh: '马克', en: 'Ma Ke', de: 'Ma Ke' }, role: 'guest', joined: '2026-08-03' },
    { id: 'claude', name: { zh: 'Claude', en: 'Claude' }, role: 'guest', kind: 'machine', agent: 'Claude Code', joined: '2026-10-12' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '找到前十个付费用户', en: 'Find our first ten paying customers', de: 'Die ersten zehn zahlenden Kunden finden' },
      summary: {
        zh: '现在 3 家在试用，0 家付费。年底前要到 10 家。',
        en: 'Three restaurants on trial, none paying. We need ten by year end.',
        de: 'Drei Restaurants testen, keins zahlt. Bis Jahresende brauchen wir zehn.',
      },
      status: 'active',
      by: 'linz',
      opened: '2026-08-05',
      updated: '2026-10-18T15:20',
      body: [
        {
          heading: { zh: '现状', en: 'Where we are', de: 'Stand' },
          text: {
            zh: '访谈了 14 家餐馆，3 家愿意试用。试用的都说好用，但没人主动问价格。',
            en: 'We interviewed 14 restaurants; 3 agreed to a trial. All three like it, but nobody has asked about price.',
            de: '14 Restaurants befragt, 3 testen. Alle drei sind zufrieden, aber niemand hat nach dem Preis gefragt.',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal', de: 'Ziel' },
          text: {
            zh: '12 月 31 日前，10 家餐馆按月付费，且连续用满一个月。',
            en: 'By 31 December, ten restaurants paying monthly and using it for a full month.',
            de: 'Bis 31. Dezember zahlen zehn Restaurants monatlich und nutzen es einen vollen Monat.',
          },
        },
      ],
      works: [
        {
          id: 's1', kind: 'doc', by: 'ma', when: '2026-10-18T15:20',
          title: { zh: '访谈记录（14 家餐馆）', en: 'Interview notes (14 restaurants)', de: 'Interviewnotizen (14 Restaurants)' },
          note: { zh: '按城区和店面大小整理', en: 'Sorted by district and size', de: 'Nach Bezirk und Größe sortiert' },
          preview: {
            excerpt: {
              zh: '“月底对账要花老板娘两个晚上。”——第 9 家，面馆，6 张桌\n“只要能拍小票就行，别让我打字。”——第 12 家，烧烤，11 张桌',
              en: '“Month-end reconciliation takes the owner two evenings.” — No. 9, noodle shop, 6 tables\n“If I can just snap the receipt, fine. Don’t make me type.” — No. 12, barbecue, 11 tables',
              de: '„Der Monatsabschluss kostet die Chefin zwei Abende.“ — Nr. 9, Nudelladen, 6 Tische\n„Wenn ich den Beleg nur fotografieren muss, gut. Bloß nicht tippen.“ — Nr. 12, Grill, 11 Tische',
            },
          },
        },
        {
          id: 's2', kind: 'table', by: 'song', when: '2026-10-14T11:00', title: pricing,
          note: { zh: '基础版 / 连锁版', en: 'Basic / chain', de: 'Basis / Kette' },
          lines: [
            { zh: '版本　　　月付　　年付', en: 'Plan        Monthly   Yearly', de: 'Tarif       Monatlich   Jährlich' },
            { zh: '基础版　　¥69　　 ¥690', en: 'Basic       ¥69       ¥690', de: 'Basis       69 ¥        690 ¥' },
            { zh: '连锁版　　¥199　　¥1,990', en: 'Chain       ¥199      ¥1,990', de: 'Kette       199 ¥       1.990 ¥' },
          ],
        },
        {
          id: 's3', kind: 'doc', by: 'linz', when: '2026-10-06T16:00',
          title: { zh: '试用协议模板', en: 'Trial agreement template', de: 'Vorlage Testvereinbarung' },
        },
        {
          id: 's4', kind: 'plan', by: 'song', when: '2026-09-29T10:00',
          title: { zh: '演示视频脚本', en: 'Demo video script', de: 'Drehbuch fürs Demovideo' },
          note: { zh: '90 秒，拍小票到出月报', en: '90 s, from receipt photo to monthly report', de: '90 s, vom Belegfoto zum Monatsbericht' },
        },
      ],
      events: [
        { by: 'ma', act: 'addWork', ref: { zh: '访谈记录（14 家餐馆）', en: 'Interview notes (14 restaurants)', de: 'Interviewnotizen (14 Restaurants)' }, when: '2026-10-18T15:20' },
        { by: 'song', act: 'update', ref: pricing, when: '2026-10-14T11:00' },
        { by: 'linz', act: 'addWork', ref: { zh: '试用协议模板', en: 'Trial agreement template', de: 'Vorlage Testvereinbarung' }, when: '2026-10-06T16:00' },
        { by: 'linz', act: 'markActive', when: '2026-08-06T09:00' },
        { by: 'linz', act: 'open', when: '2026-08-05T21:00' },
      ],
      notes: [
        { by: 'ma', when: '2026-10-18T15:30', text: { zh: '第 9 家和第 12 家都说，只要能拍小票就愿意试。下周先去这两家。', en: 'Nos. 9 and 12 both said they’d try it as long as they can just photograph receipts. Let’s start with those two next week.', de: 'Nr. 9 und Nr. 12 wollen es probieren, solange sie nur Belege fotografieren müssen. Nächste Woche zuerst zu diesen beiden.' } },
        { by: 'linz', when: '2026-10-18T17:05', text: { zh: '同意。去之前把演示视频剪到 60 秒以内。', en: 'Agreed. Cut the demo video to under 60 seconds before we go.', de: 'Einverstanden. Vorher das Demovideo auf unter 60 Sekunden kürzen.' } },
        { by: 'claude', when: '2026-10-18T20:40', text: { zh: '我把 14 份访谈按"愿意试用的理由"归了类：拍小票 6 家、月底对账 4 家、其他 4 家。第 9 家和第 12 家都在"拍小票"里，先去这两家是对的。', en: 'I grouped the 14 interviews by why they would try it: receipt photos 6, month-end reconciliation 4, other 4. Nos. 9 and 12 are both in the receipt-photo group, so starting with them makes sense.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '定价：按月还是按年', en: 'Pricing: monthly or yearly?', de: 'Preismodell: monatlich oder jährlich?' },
      summary: {
        zh: '小餐馆现金流紧，年付可能吓跑人；月付流失率又高。',
        en: 'Small restaurants are short on cash, so yearly may scare them off; monthly churns more.',
        de: 'Kleine Restaurants haben wenig Liquidität, jährlich schreckt ab; monatlich springen mehr ab.',
      },
      status: 'active', by: 'song', opened: '2026-09-20', updated: '2026-10-17T23:10',
      body: [], works: [], workCount: 2, events: [],
    },
    {
      id: '0003',
      title: { zh: '注册公司与开对公账户', en: 'Register the company and open a business account', de: 'Firma anmelden und Geschäftskonto eröffnen' },
      summary: {
        zh: '已在 9 月完成，营业执照和开户回单都在成果里。',
        en: 'Done in September; the licence and bank receipt are in the results.',
        de: 'Im September erledigt; Gewerbeschein und Kontobestätigung liegen bei den Ergebnissen.',
      },
      status: 'solved', by: 'linz', opened: '2026-08-02', updated: '2026-09-12',
      body: [], works: [], workCount: 4, events: [],
    },
    {
      id: '0004',
      title: { zh: '第一版能不能离线用', en: 'Should the first version work offline?', de: 'Soll die erste Version offline funktionieren?' },
      summary: {
        zh: '后厨信号差。离线要多做三周，值不值得？',
        en: 'Kitchens have poor signal. Offline costs three more weeks. Worth it?',
        de: 'In Küchen ist der Empfang schlecht. Offline kostet drei Wochen mehr. Lohnt sich das?',
      },
      status: 'open', by: 'ma', opened: '2026-10-10', updated: '2026-10-10',
      body: [], works: [], workCount: 1, events: [],
    },
  ],
  recent: [
    { by: 'claude', problem: '0004', act: 'propose', ref: offlinePlan, when: '2026-10-18T21:05' },
    { by: 'ma', problem: '0001', act: 'addWork', ref: { zh: '访谈记录（14 家餐馆）', en: 'Interview notes (14 restaurants)', de: 'Interviewnotizen (14 Restaurants)' }, when: '2026-10-18T15:20' },
    { by: 'ma', problem: '0002', act: 'propose', ref: cutPrice, when: '2026-10-17T23:10' },
    { by: 'song', problem: '0001', act: 'update', ref: pricing, when: '2026-10-14T11:00' },
    { by: 'ma', problem: '0004', act: 'openProblem', ref: { zh: '第一版能不能离线用', en: 'Should the first version work offline?', de: 'Soll die erste Version offline funktionieren?' }, when: '2026-10-10T13:00' },
  ],
  proposals: [
    {
      id: 'p1',
      title: cutPrice,
      by: 'ma',
      problem: '0002',
      when: '2026-10-17T23:10',
      reason: {
        zh: '14 家里有 9 家说现在用的记账软件不超过 50 元。69 元一出口就被比下去了。连锁版不动。',
        en: 'Nine of the fourteen say their current bookkeeping tool costs under ¥50. At ¥69 we lose the comparison straight away. Chain plan unchanged.',
        de: 'Neun von vierzehn zahlen für ihre jetzige Buchhaltung unter 50 ¥. Mit 69 ¥ verlieren wir den Vergleich sofort. Kettentarif bleibt.',
      },
      files: [
        {
          title: pricing,
          kind: 'table',
          lines: [
            { kind: 'same', text: { zh: '版本　　　月付　　年付', en: 'Plan        Monthly   Yearly', de: 'Tarif       Monatlich   Jährlich' } },
            { kind: 'del', text: { zh: '基础版　　¥69　　 ¥690', en: 'Basic       ¥69       ¥690', de: 'Basis       69 ¥        690 ¥' } },
            { kind: 'add', text: { zh: '基础版　　¥49　　 ¥490', en: 'Basic       ¥49       ¥490', de: 'Basis       49 ¥        490 ¥' } },
            { kind: 'same', text: { zh: '连锁版　　¥199　　¥1,990', en: 'Chain       ¥199      ¥1,990', de: 'Kette       199 ¥       1.990 ¥' } },
            { kind: 'add', text: { zh: '前三个月半价，仅限前十家。', en: 'Half price for three months, first ten customers only.', de: 'Drei Monate zum halben Preis, nur für die ersten zehn Kunden.' } },
          ],
        },
      ],
    },
    {
      id: 'p2',
      title: offlinePlan,
      by: 'claude',
      problem: '0004',
      when: '2026-10-18T21:05',
      reason: {
        zh: '14 家里有 11 家后厨信号差，但都说月底才对账。所以第一版只要能离线记、联网后补传，不必做离线查询，大约省两周。',
        en: 'Eleven of the fourteen kitchens have poor signal, but all reconcile only at month end. So version one just needs to record offline and upload later; no offline queries. Saves about two weeks.',
      },
      files: [
        {
          title: { zh: '离线方案', en: 'Offline plan' },
          kind: 'plan',
          lines: [
            { kind: 'add', text: { zh: '第一版：断网时照常记账，存在手机里。', en: 'Version one: keep recording when offline, stored on the phone.' } },
            { kind: 'add', text: { zh: '联网后自动补传，按记账时间排好。', en: 'Upload automatically once online, ordered by when each entry was made.' } },
            { kind: 'add', text: { zh: '离线时不能查月报，界面上说清楚。', en: 'Monthly reports need a connection; the screen says so.' } },
          ],
        },
      ],
    },
  ],
  board: [
    {
      id: 'b1', by: 'linz', when: '2026-10-17T09:00',
      title: { zh: '下周二起，周会改到早上九点', en: 'Weekly meeting moves to 9 am from next Tuesday', de: 'Ab nächstem Dienstag ist das Wochentreffen um 9 Uhr' },
      text: {
        zh: '中午要跑餐馆，只有早上能凑齐三个人。会上只看三件事：付费用户数、流失原因、下周谁去拜访哪几家。',
        en: 'We visit restaurants at lunchtime, so mornings are the only time all three of us are free. We look at three things only: paying customers, why people dropped out, and who visits which restaurants next week.',
        de: 'Mittags sind wir in den Restaurants, nur morgens sind alle drei da. Wir schauen nur auf drei Dinge: zahlende Kunden, Gründe für Absprünge und wer nächste Woche welche Restaurants besucht.',
      },
      replies: [{ by: 'ma', when: '2026-10-17T09:40', text: { zh: '好，我九点前把访谈记录整理完。', en: 'OK. I’ll have the interview notes tidied up by nine.', de: 'Gut, bis neun habe ich die Interviewnotizen sortiert.' } }],
    },
  ],
  decided: [
    { title: { zh: '用拍小票代替手动录入', en: 'Receipt photos instead of manual entry', de: 'Belegfotos statt Handeingabe' }, by: 'ma', result: 'merged', when: '2026-09-02' },
  ],
}
