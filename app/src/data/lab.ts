// 液滴论文小组：实验室里四个人合写一篇论文。当前用户是茶客（博士生叶舟）。
import type { House } from './types'

const section3 = { zh: '第 3 节草稿', en: 'Draft of Section 3', de: 'Entwurf von Abschnitt 3' }
const conditions = { zh: '三月与十月的实验条件对照', en: 'March vs October conditions', de: 'Versuchsbedingungen März vs. Oktober' }
const honest = { zh: '第 3 节：写明实验条件和复现差异', en: 'Section 3: state the conditions and the reproduction gap', de: 'Abschnitt 3: Bedingungen und Reproduktionsabweichung offenlegen' }

export const lab: House = {
  id: 'lab',
  name: { zh: '液滴论文小组', en: 'Droplet Paper Group', de: 'Tröpfchen-Schreibgruppe' },
  nameLines: { zh: ['液滴论文', '小组'], en: ['Droplet', 'Paper Group'], de: ['Tröpfchen-', 'Schreibgruppe'] },
  sign: {
    zh: '四个人，一篇论文，一月投稿。每个数字都要能复现。',
    en: 'Four people, one paper, submitted in January. Every number has to be reproducible.',
    de: 'Vier Leute, ein Paper, Einreichung im Januar. Jede Zahl muss reproduzierbar sein.',
  },
  founded: '2026-07-01',
  capacity: 15,
  me: 'ye',
  members: [
    { id: 'sunl', name: { zh: '孙岚', en: 'Sun Lan', de: 'Sun Lan' }, role: 'host', joined: '2026-07-01' },
    { id: 'jiang', name: { zh: '江河', en: 'Jiang He', de: 'Jiang He' }, role: 'steward', joined: '2026-07-01' },
    { id: 'ye', name: { zh: '叶舟', en: 'Ye Zhou', de: 'Ye Zhou' }, role: 'guest', joined: '2026-07-02' },
    { id: 'bai', name: { zh: '白杨', en: 'Bai Yang', de: 'Bai Yang' }, role: 'guest', joined: '2026-07-02' },
  ],
  problems: [
    {
      id: '0001',
      title: {
        zh: '第 3 节的实验为什么复现不了',
        en: 'Why can’t we reproduce the Section 3 experiment?',
        de: 'Warum lässt sich das Experiment aus Abschnitt 3 nicht reproduzieren?',
      },
      summary: {
        zh: '三月测得液滴直径 42 μm，十月重做三次都在 51 μm 左右。',
        en: 'In March the droplets measured 42 μm; three repeats in October all came out around 51 μm.',
        de: 'Im März maßen die Tröpfchen 42 μm; drei Wiederholungen im Oktober lagen alle bei etwa 51 μm.',
      },
      status: 'active',
      by: 'ye',
      opened: '2026-10-05',
      updated: '2026-10-18T11:30',
      body: [
        {
          heading: { zh: '现状', en: 'Now', de: 'Stand' },
          text: {
            zh: '三月的数据里液滴直径是 42 ± 3 μm，十月重做三次都是 51 μm 左右。芯片、泵和配方，记录上都说是一样的。',
            en: 'March gave 42 ± 3 μm; three October repeats all gave about 51 μm. According to the records, chip, pump and recipe were all the same.',
            de: 'Im März 42 ± 3 μm, im Oktober dreimal etwa 51 μm. Laut Protokoll waren Chip, Pumpe und Rezeptur gleich.',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal', de: 'Ziel' },
          text: {
            zh: '找到差异的来源；找不到，就在论文里如实写出条件。',
            en: 'Find where the difference comes from; failing that, state the conditions honestly in the paper.',
            de: 'Die Ursache finden; wenn nicht, die Bedingungen im Paper offen angeben.',
          },
        },
        {
          heading: { zh: '约束', en: 'Limits', de: 'Rahmen' },
          text: {
            zh: '一月投稿前要定。新芯片要六周才到，只能用手上的两片。',
            en: 'Settled before the January submission. New chips take six weeks, so only the two we have.',
            de: 'Vor der Einreichung im Januar klären. Neue Chips brauchen sechs Wochen, also nur die zwei vorhandenen.',
          },
        },
      ],
      works: [
        {
          id: 'l1', kind: 'plan', by: 'jiang', when: '2026-10-18T11:30', title: conditions,
          note: { zh: '逐项对照实验记录本', en: 'Line by line against the lab notebooks', de: 'Punkt für Punkt gegen die Laborbücher' },
          preview: {
            excerpt: {
              zh: '芯片：A-07 → A-07（同一片）\n连续相黏度：9.8 → 12.1 mPa·s（十月用了新批次的表面活性剂）\n室温：21 °C → 17 °C',
              en: 'Chip: A-07 → A-07 (the same one)\nContinuous-phase viscosity: 9.8 → 12.1 mPa·s (October used a new surfactant batch)\nRoom temperature: 21 °C → 17 °C',
              de: 'Chip: A-07 → A-07 (derselbe)\nViskosität der kontinuierlichen Phase: 9,8 → 12,1 mPa·s (im Oktober neue Tensid-Charge)\nRaumtemperatur: 21 °C → 17 °C',
            },
          },
        },
        {
          id: 'l2', kind: 'doc', by: 'ye', when: '2026-10-14T16:00', title: section3,
          history: [
            {
              by: 'ye', when: '2026-10-12T21:30', act: 'add',
              diff: [
                { kind: 'add', text: { zh: '3 实验', en: '3 Experiment', de: '3 Experiment' } },
                { kind: 'add', text: { zh: '液滴在 T 型通道中生成，分散相流速 2 μl/min。', en: 'Droplets were formed in a T-junction at a dispersed-phase flow rate of 2 μl/min.', de: 'Die Tröpfchen entstanden in einer T-Kreuzung bei 2 μl/min Durchfluss der dispersen Phase.' } },
                { kind: 'add', text: { zh: '液滴直径为 42 ± 3 μm（n = 120）。', en: 'Droplet diameter was 42 ± 3 μm (n = 120).', de: 'Der Tröpfchendurchmesser betrug 42 ± 3 μm (n = 120).' } },
              ],
            },
            {
              by: 'jiang', when: '2026-10-13T10:15', act: 'update',
              diff: [
                { kind: 'same', text: { zh: '3 实验', en: '3 Experiment', de: '3 Experiment' } },
                { kind: 'del', text: { zh: '液滴在 T 型通道中生成，分散相流速 2 μl/min。', en: 'Droplets were formed in a T-junction at a dispersed-phase flow rate of 2 μl/min.', de: 'Die Tröpfchen entstanden in einer T-Kreuzung bei 2 μl/min Durchfluss der dispersen Phase.' } },
                { kind: 'add', text: { zh: '液滴在 T 型通道中生成，分散相流速 2 μL/min。', en: 'Droplets were formed in a T-junction at a dispersed-phase flow rate of 2 μL/min.', de: 'Die Tröpfchen entstanden in einer T-Kreuzung bei 2 μL/min Durchfluss der dispersen Phase.' } },
                { kind: 'same', text: { zh: '液滴直径为 42 ± 3 μm（n = 120）。', en: 'Droplet diameter was 42 ± 3 μm (n = 120).', de: 'Der Tröpfchendurchmesser betrug 42 ± 3 μm (n = 120).' } },
              ],
            },
            {
              by: 'ye', when: '2026-10-14T16:00', act: 'update',
              diff: [
                { kind: 'same', text: { zh: '液滴直径为 42 ± 3 μm（n = 120）。', en: 'Droplet diameter was 42 ± 3 μm (n = 120).', de: 'Der Tröpfchendurchmesser betrug 42 ± 3 μm (n = 120).' } },
                { kind: 'add', text: { zh: '图 3 给出直径分布。', en: 'Figure 3 shows the size distribution.', de: 'Abbildung 3 zeigt die Größenverteilung.' } },
              ],
            },
          ],
          note: { zh: '实验部分，约 600 字', en: 'Methods, about 600 words', de: 'Methodenteil, etwa 600 Wörter' },
          lines: [
            { zh: '3 实验', en: '3 Experiment', de: '3 Experiment' },
            { zh: '液滴在 T 型通道中生成，分散相流速 2 μL/min。', en: 'Droplets were formed in a T-junction at a dispersed-phase flow rate of 2 μL/min.', de: 'Die Tröpfchen entstanden in einer T-Kreuzung bei 2 μL/min Durchfluss der dispersen Phase.' },
            { zh: '液滴直径为 42 ± 3 μm（n = 120）。', en: 'Droplet diameter was 42 ± 3 μm (n = 120).', de: 'Der Tröpfchendurchmesser betrug 42 ± 3 μm (n = 120).' },
            { zh: '图 3 给出直径分布。', en: 'Figure 3 shows the size distribution.', de: 'Abbildung 3 zeigt die Größenverteilung.' },
          ],
        },
        {
          id: 'l3', kind: 'table', by: 'bai', when: '2026-10-11T19:00',
          title: { zh: '复现记录（三次）', en: 'Reproduction log (three runs)', de: 'Reproduktionsprotokoll (drei Läufe)' },
          note: { zh: '每次 120 个液滴', en: '120 droplets per run', de: '120 Tröpfchen pro Lauf' },
        },
        {
          id: 'l4', kind: 'image', by: 'ye', when: '2026-10-09T22:00',
          title: { zh: '图 3：直径分布（三月 / 十月）', en: 'Figure 3: size distribution (March / October)', de: 'Abbildung 3: Größenverteilung (März / Oktober)' },
        },
      ],
      events: [
        { by: 'jiang', act: 'addWork', ref: conditions, when: '2026-10-18T11:30' },
        { by: 'ye', act: 'propose', ref: honest, when: '2026-10-18T09:15' },
        { by: 'ye', act: 'addWork', ref: section3, when: '2026-10-14T16:00' },
        { by: 'sunl', act: 'markActive', when: '2026-10-06T09:00' },
        { by: 'ye', act: 'open', when: '2026-10-05T20:00' },
      ],
      // 留言按进入茶馆的先后排列，不按时间戳：叶舟那条是在火车上离线写的，晚了一天才同步进来。
      notes: [
        { by: 'bai', when: '2026-10-16T10:20', text: { zh: '十月配的油用的是新到的一批表面活性剂。', en: 'The oil we mixed in October used the new batch of surfactant.', de: 'Das Öl im Oktober war mit der neuen Tensid-Charge angesetzt.' } },
        { by: 'jiang', when: '2026-10-17T14:05', text: { zh: '黏度高了两成，直径偏大说得通。下周用旧批次再做一次。', en: 'Viscosity is up 20%, so larger droplets make sense. Next week we repeat it with the old batch.', de: 'Die Viskosität ist 20 % höher, größere Tröpfchen passen dazu. Nächste Woche wiederholen wir mit der alten Charge.' } },
        { by: 'ye', when: '2026-10-16T21:40', text: { zh: '（在火车上写的）三月记录里室温是 21 度，现在实验室只有 17 度，可能也有关系。', en: '(Written on the train) The March notebook says 21 °C; the lab is only 17 °C now. That may matter too.', de: '(Im Zug geschrieben) Laut Märzprotokoll waren es 21 °C, jetzt hat das Labor nur 17 °C. Das könnte auch eine Rolle spielen.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '图 2 要不要换成对数坐标', en: 'Should Figure 2 use a log scale?', de: 'Soll Abbildung 2 logarithmisch skaliert sein?' },
      summary: {
        zh: '流速跨了三个数量级，线性坐标下小流速的点挤在一起。',
        en: 'Flow rates span three orders of magnitude; on a linear axis the low-flow points pile up.',
        de: 'Die Durchflüsse reichen über drei Größenordnungen; linear skaliert ballen sich die kleinen Werte.',
      },
      status: 'open', by: 'bai', opened: '2026-10-13', updated: '2026-10-13',
      body: [], works: [], workCount: 2, events: [],
    },
    {
      id: '0003',
      title: { zh: '讨论部分谁来写', en: 'Who writes the Discussion?', de: 'Wer schreibt die Diskussion?' },
      summary: {
        zh: '江河先写框架，孙老师改。要等第 3 节的问题定下来。',
        en: 'Jiang He drafts the outline, Prof. Sun revises. It waits on the Section 3 question.',
        de: 'Jiang He entwirft die Gliederung, Prof. Sun überarbeitet. Wartet auf die Frage zu Abschnitt 3.',
      },
      status: 'active', by: 'sunl', opened: '2026-09-22', updated: '2026-10-08',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0004',
      title: { zh: '原始数据整理上传', en: 'Tidy up and upload the raw data', de: 'Rohdaten ordnen und hochladen' },
      summary: {
        zh: '按期刊要求整理好，已经上传到数据仓库，拿到了编号。',
        en: 'Organised to the journal’s requirements, uploaded to the repository, identifier received.',
        de: 'Nach Vorgaben der Zeitschrift geordnet, ins Repositorium hochgeladen, Kennung erhalten.',
      },
      status: 'solved', by: 'jiang', opened: '2026-08-01', updated: '2026-09-02',
      body: [], works: [], workCount: 2, events: [],
    },
  ],
  recent: [
    { by: 'jiang', problem: '0001', act: 'addWork', ref: conditions, when: '2026-10-18T11:30' },
    { by: 'ye', problem: '0001', act: 'propose', ref: honest, when: '2026-10-18T09:15' },
    { by: 'ye', problem: '0001', act: 'addWork', ref: section3, when: '2026-10-14T16:00' },
    { by: 'bai', problem: '0002', act: 'openProblem', ref: { zh: '图 2 要不要换成对数坐标', en: 'Should Figure 2 use a log scale?', de: 'Soll Abbildung 2 logarithmisch skaliert sein?' }, when: '2026-10-13T15:00' },
  ],
  proposals: [
    {
      id: 'p1',
      title: honest,
      by: 'ye',
      problem: '0001',
      when: '2026-10-18T09:15',
      reason: {
        zh: '不管最后找没找到原因，审稿人都该看到两次的条件。先把能确定的写进去。',
        en: 'Whether or not we find the cause, reviewers should see the conditions of both runs. Let’s write down what we know for sure.',
        de: 'Ob wir die Ursache finden oder nicht, die Gutachter sollten die Bedingungen beider Läufe sehen. Schreiben wir auf, was sicher ist.',
      },
      files: [
        {
          title: section3,
          kind: 'doc',
          lines: [
            { kind: 'same', text: { zh: '3 实验', en: '3 Experiment', de: '3 Experiment' } },
            { kind: 'same', text: { zh: '液滴在 T 型通道中生成，分散相流速 2 μL/min。', en: 'Droplets were formed in a T-junction at a dispersed-phase flow rate of 2 μL/min.', de: 'Die Tröpfchen entstanden in einer T-Kreuzung bei 2 μL/min Durchfluss der dispersen Phase.' } },
            { kind: 'del', text: { zh: '液滴直径为 42 ± 3 μm（n = 120）。', en: 'Droplet diameter was 42 ± 3 μm (n = 120).', de: 'Der Tröpfchendurchmesser betrug 42 ± 3 μm (n = 120).' } },
            { kind: 'add', text: { zh: '三月测得液滴直径为 42 ± 3 μm（n = 120，室温 21 °C）。', en: 'In March the droplet diameter was 42 ± 3 μm (n = 120, room temperature 21 °C).', de: 'Im März betrug der Durchmesser 42 ± 3 μm (n = 120, Raumtemperatur 21 °C).' } },
            { kind: 'add', text: { zh: '十月用新批次表面活性剂重复实验，直径为 51 ± 4 μm；差异与连续相黏度升高一致（见补充材料 S2）。', en: 'Repeated in October with a new surfactant batch, the diameter was 51 ± 4 μm, consistent with the higher continuous-phase viscosity (see Supplement S2).', de: 'Im Oktober mit neuer Tensid-Charge wiederholt: 51 ± 4 μm, passend zur höheren Viskosität der kontinuierlichen Phase (siehe Anhang S2).' } },
            { kind: 'same', text: { zh: '图 3 给出直径分布。', en: 'Figure 3 shows the size distribution.', de: 'Abbildung 3 zeigt die Größenverteilung.' } },
          ],
        },
      ],
    },
  ],
  decided: [
    { title: { zh: '原始数据按期刊格式整理', en: 'Raw data in the journal’s format', de: 'Rohdaten im Format der Zeitschrift' }, by: 'jiang', result: 'merged', when: '2026-09-02' },
  ],
  board: [
    {
      id: 'b1', by: 'sunl', when: '2026-10-16T08:30',
      title: { zh: '组会改到周四下午两点', en: 'Group meeting moves to Thursday 2 pm', de: 'Gruppentreffen jetzt donnerstags um 14 Uhr' },
      text: {
        zh: '每人带上自己负责的那一节，打印出来。第 3 节先讨论。',
        en: 'Everyone brings a printout of the section they’re responsible for. Section 3 goes first.',
        de: 'Alle bringen ihren Abschnitt ausgedruckt mit. Abschnitt 3 kommt zuerst.',
      },
      replies: [
        { by: 'ye', when: '2026-10-16T09:02', text: { zh: '好。我把复现差异的提案先交上来，周四一起看。', en: 'OK. I’ll submit my proposal on the reproduction gap first so we can look at it on Thursday.', de: 'Gut. Ich reiche meinen Vorschlag zur Abweichung vorher ein, dann sehen wir ihn uns am Donnerstag an.' } },
      ],
    },
  ],
}
