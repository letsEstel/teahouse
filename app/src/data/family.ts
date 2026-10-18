// 爸妈的老房子：三家兄妹一起给父母改造老房子。当前用户是伙计（二哥何磊）。
// 德语暂缺，界面会退到英语。
import type { House } from './types'

const split = { zh: '费用分摊表', en: 'Cost split' }
const plan = { zh: '卫生间改造方案（第二版）', en: 'Bathroom plan (2nd version)' }
const changeSplit = { zh: '分摊改为四成、四成、两成', en: 'Change the split to 40 / 40 / 20' }

export const family: House = {
  id: 'family',
  name: { zh: '爸妈的老房子', en: 'Mum and Dad’s Old Flat' },
  nameLines: { zh: ['爸妈的', '老房子'], en: ['Mum and Dad’s', 'Old Flat'] },
  sign: {
    zh: '入冬之前，让爸妈住进不漏风、不用爬高的家。钱和力气，三家一起出。',
    en: 'Before winter: a home for Mum and Dad with no draughts and no climbing. Money and effort from all three families.',
  },
  founded: '2026-09-01',
  capacity: 15,
  me: 'lei',
  members: [
    { id: 'jing', name: { zh: '何静', en: 'He Jing' }, role: 'boss', joined: '2026-09-01' },
    { id: 'lei', name: { zh: '何磊', en: 'He Lei' }, role: 'heir', joined: '2026-09-01' },
    { id: 'ping', name: { zh: '何平', en: 'He Ping' }, role: 'guest', joined: '2026-09-01' },
    { id: 'mum', name: { zh: '妈妈', en: 'Mum' }, role: 'guest', joined: '2026-09-02' },
    { id: 'dad', name: { zh: '爸爸', en: 'Dad' }, role: 'guest', joined: '2026-09-02' },
    { id: 'wei', name: { zh: '李薇', en: 'Li Wei' }, role: 'guest', joined: '2026-09-03' },
    { id: 'yu', name: { zh: '小雨', en: 'Xiaoyu' }, role: 'guest', joined: '2026-09-10' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '卫生间改成爸妈能放心用的', en: 'Make the bathroom safe for Mum and Dad' },
      summary: {
        zh: '蹲坑、没扶手、门口有台阶。爸去年冬天在这里滑倒过一次。',
        en: 'A squat toilet, no grab rails, a step at the door. Dad slipped here last winter.',
      },
      status: 'active',
      by: 'jing',
      opened: '2026-09-01',
      updated: '2026-10-18T16:40',
      body: [
        {
          heading: { zh: '现状', en: 'Now' },
          text: {
            zh: '蹲坑，没有扶手，门口一级 12 厘米的台阶，热水器装在门背后。爸去年一月在这里滑倒过。',
            en: 'Squat toilet, no rails, a 12 cm step at the door, the water heater behind the door. Dad slipped here last January.',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal' },
          text: {
            zh: '坐便、淋浴区不留门槛、两处扶手、防滑地砖。十二月前完工。',
            en: 'A sit-down toilet, a step-free shower, two grab rails, non-slip tiles. Finished before December.',
          },
        },
        {
          heading: { zh: '约束', en: 'Limits' },
          text: {
            zh: '预算三万以内；施工期间爸妈住小姨家，最多三周。',
            en: 'Under ¥30,000; Mum and Dad stay at their sister’s during the work, three weeks at most.',
          },
        },
      ],
      works: [
        {
          id: 'f1', kind: 'plan', by: 'jing', when: '2026-10-18T16:40', title: plan,
          note: { zh: '对比了三家施工队的建议', en: 'Compared what three builders suggested' },
          preview: {
            excerpt: {
              zh: '拆掉门口台阶，地面整体找坡到地漏。\n蹲坑改坐便，旁边装一处竖扶手；淋浴区装一处横扶手，离地 80 厘米。\n热水器移到门对面的墙上，开关做低。',
              en: 'Remove the step at the door; slope the whole floor to the drain.\nReplace the squat toilet with a seat and a vertical rail beside it; a horizontal rail in the shower, 80 cm up.\nMove the water heater to the opposite wall, with the switch set low.',
            },
          },
        },
        {
          id: 'f2', kind: 'bill', by: 'lei', when: '2026-10-15T21:00', title: split,
          note: { zh: '三家平摊', en: 'Split three ways' },
          lines: [
            { zh: '项目　　　　金额', en: 'Item            Amount' },
            { zh: '卫生间改造　¥28,000', en: 'Bathroom        ¥28,000' },
            { zh: '何静　　　　三分之一', en: 'He Jing         one third' },
            { zh: '何磊　　　　三分之一', en: 'He Lei          one third' },
            { zh: '何平　　　　三分之一', en: 'He Ping         one third' },
          ],
        },
        {
          id: 'f3', kind: 'bill', by: 'ping', when: '2026-10-11T12:00',
          title: { zh: '三家施工队报价', en: 'Quotes from three builders' },
          note: { zh: '最便宜的那家不包垃圾清运', en: 'The cheapest one doesn’t take away rubble' },
        },
        {
          id: 'f4', kind: 'image', by: 'yu', when: '2026-10-06T18:00',
          title: { zh: '现状照片和尺寸', en: 'Photos and measurements' },
        },
        {
          id: 'f5', kind: 'doc', by: 'wei', when: '2026-09-28T20:00',
          title: { zh: '扶手和坐便选购清单', en: 'Shopping list: rails and toilet' },
          note: { zh: '附适老化标准', en: 'With the accessibility standard attached' },
        },
      ],
      events: [
        { by: 'jing', act: 'addWork', ref: plan, when: '2026-10-18T16:40' },
        { by: 'ping', act: 'propose', ref: changeSplit, when: '2026-10-17T23:30' },
        { by: 'lei', act: 'addWork', ref: split, when: '2026-10-15T21:00' },
        { by: 'jing', act: 'markActive', when: '2026-09-05T10:00' },
        { by: 'jing', act: 'open', when: '2026-09-01T20:00' },
      ],
      notes: [
        { by: 'mum', when: '2026-10-16T09:20', text: { zh: '扶手别装太高，我够不着。', en: 'Don’t put the rails too high, I can’t reach.' } },
        { by: 'lei', when: '2026-10-16T21:05', text: { zh: '按 80 厘米装。妈，下回我带尺子来，您站着比一下。', en: 'We’ll fit them at 80 cm. Mum, next time I’ll bring a tape and you can try the height.' } },
        { by: 'yu', when: '2026-10-18T14:10', text: { zh: '防滑砖我看了三款，照片传到“现状照片”里了，奶奶喜欢浅色那款。', en: 'I looked at three non-slip tiles and added photos to “Photos and measurements”. Grandma likes the light one.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '三家怎么分摊费用', en: 'How the three families split the cost' },
      summary: {
        zh: '平摊最简单，但老三今年刚换工作。',
        en: 'Splitting evenly is simplest, but the youngest just changed jobs.',
      },
      status: 'active', by: 'lei', opened: '2026-10-12', updated: '2026-10-17T23:30',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0003',
      title: { zh: '找一个靠谱的施工队', en: 'Find a builder we can trust' },
      summary: {
        zh: '三家报价都在成果里。要看过他们做过的老房子再定。',
        en: 'All three quotes are in the results. Visit a flat they’ve done before deciding.',
      },
      status: 'active', by: 'ping', opened: '2026-09-20', updated: '2026-10-11',
      body: [], works: [], workCount: 3, events: [],
    },
    {
      id: '0004',
      title: { zh: '施工期间爸妈住哪', en: 'Where Mum and Dad stay during the work' },
      summary: {
        zh: '小姨家有空房，已经说好了，11 月 10 日搬过去。',
        en: 'Their sister has a spare room. Agreed: they move on 10 November.',
      },
      status: 'solved', by: 'wei', opened: '2026-09-15', updated: '2026-10-02',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0005',
      title: { zh: '妈想留下的旧家具', en: 'The old furniture Mum wants to keep' },
      summary: {
        zh: '樟木箱、缝纫机、外公做的方凳。装修时放哪？',
        en: 'The camphor chest, the sewing machine, the stool Grandpa made. Where do they go during the work?',
      },
      status: 'open', by: 'mum', opened: '2026-10-14', updated: '2026-10-14',
      body: [], works: [], workCount: 0, events: [],
    },
  ],
  recent: [
    { by: 'jing', problem: '0001', act: 'addWork', ref: plan, when: '2026-10-18T16:40' },
    { by: 'ping', problem: '0002', act: 'propose', ref: changeSplit, when: '2026-10-17T23:30' },
    { by: 'lei', problem: '0001', act: 'addWork', ref: split, when: '2026-10-15T21:00' },
    { by: 'mum', problem: '0005', act: 'openProblem', ref: { zh: '妈想留下的旧家具', en: 'The old furniture Mum wants to keep' }, when: '2026-10-14T10:00' },
  ],
  proposals: [
    {
      id: 'p1',
      title: changeSplit,
      by: 'ping',
      problem: '0002',
      when: '2026-10-17T23:30',
      reason: {
        zh: '我九月刚换工作，试用期工资少一截。今年我先出两成，明年三月前把差额补给大姐和二哥。',
        en: 'I changed jobs in September and earn less during probation. Let me pay 20% this year and make up the difference to my brother and sister by next March.',
      },
      files: [
        {
          title: split,
          kind: 'bill',
          lines: [
            { kind: 'same', text: { zh: '项目　　　　金额', en: 'Item            Amount' } },
            { kind: 'same', text: { zh: '卫生间改造　¥28,000', en: 'Bathroom        ¥28,000' } },
            { kind: 'del', text: { zh: '何静　　　　三分之一', en: 'He Jing         one third' } },
            { kind: 'del', text: { zh: '何磊　　　　三分之一', en: 'He Lei          one third' } },
            { kind: 'del', text: { zh: '何平　　　　三分之一', en: 'He Ping         one third' } },
            { kind: 'add', text: { zh: '何静　　　　40%', en: 'He Jing         40%' } },
            { kind: 'add', text: { zh: '何磊　　　　40%', en: 'He Lei          40%' } },
            { kind: 'add', text: { zh: '何平　　　　20%（明年三月前补齐差额）', en: 'He Ping         20% (difference repaid by March)' } },
          ],
        },
      ],
    },
  ],
  decided: [
    { title: { zh: '施工期间住小姨家', en: 'Stay with their sister during the work' }, by: 'wei', result: 'merged', when: '2026-10-02' },
  ],
  board: [
    {
      id: 'b2', by: 'jing', when: '2026-10-18T19:30',
      title: { zh: '11 月 2 日（周日）家庭会，定施工队', en: 'Family meeting Sunday 2 November: choose the builder' },
      text: {
        zh: '下午三点在爸妈家。请大家先看完三家报价和施工队做过的房子照片。能来的在下面回一句。',
        en: '3 pm at Mum and Dad’s. Please look through the three quotes and the photos of their past work first. Reply below if you can come.',
      },
      replies: [
        { by: 'mum', when: '2026-10-18T19:52', text: { zh: '我包饺子。', en: 'I’ll make dumplings.' } },
        { by: 'ping', when: '2026-10-18T20:05', text: { zh: '我五点前到。', en: 'I’ll be there before five.' } },
      ],
    },
    {
      id: 'b1', by: 'lei', when: '2026-10-02T21:00',
      title: { zh: '爸妈 11 月 10 日起住小姨家', en: 'Mum and Dad move to their sister’s on 10 November' },
      text: {
        zh: '我开车送。冬衣和药先打包好，缺什么我再回来拿。',
        en: 'I’ll drive them. Pack winter clothes and medicine first; I’ll fetch anything missing.',
      },
      replies: [{ by: 'dad', when: '2026-10-03T08:15', text: { zh: '被子我自己带。', en: 'I’ll bring my own quilt.' } }],
    },
  ],
}
