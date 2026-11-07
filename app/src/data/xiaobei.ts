// 小北街：核桃园里的第七个茶摊子（docs/NAMES.zh.md）。几个老同学春节回家，带朋友 Sebastian 走一天：
// 五个地方、几样吃的，一张纸写清楚。当前用户是老板（马晓）。
// 《五处牌匾》只填已知的两块，其余三块写“还没抄回来”，不编。
import type { House } from './types'

const route = { zh: '凉州一日路线', en: 'One day in Liangzhou', de: 'Ein Tag in Liangzhou' }
const plaques = { zh: '五处牌匾', en: 'Five plaques', de: 'Fünf Tafeln' }
const forSeb = { zh: '给塞巴斯蒂安的一页介绍', en: 'One page for Sebastian', de: 'Eine Seite für Sebastian' }
const buyun = { zh: '雷台的“步云”', en: 'Leitai: “Bu Yun”', de: 'Leitai: „Bu Yun“' }
const morningBell = { zh: '路线：大云寺挪到上午，去听晓钟', en: 'Route: move Dayun Temple to the morning, for the dawn bell', de: 'Route: Dayun-Tempel auf den Morgen legen, wegen der Morgenglocke' }
const votes = { zh: '面皮子投票', en: 'Mianpizi vote', de: 'Abstimmung: Mianpizi' }

export const xiaobei: House = {
  id: 'xiaobei',
  name: { zh: '小北街', en: 'Xiaobei Street', de: 'Xiaobei-Straße' },
  nameLines: { zh: ['小北街'], en: ['Xiaobei', 'Street'], de: ['Xiaobei-', 'Straße'] },
  sign: {
    zh: '春节回凉州，带塞巴斯蒂安走一天。五个地方，几样吃的，一张纸写清楚。',
    en: 'Home to Liangzhou for Spring Festival, one day to show Sebastian around. Five places, a few things to eat, all on one sheet of paper.',
    de: 'Zum Frühlingsfest zurück nach Liangzhou, ein Tag, um Sebastian alles zu zeigen. Fünf Orte, ein paar Gerichte, alles auf einem Blatt.',
  },
  founded: '2026-10-02',
  capacity: 15,
  me: 'ma',
  members: [
    { id: 'ma', name: { zh: '马晓', en: 'Ma Xiao', de: 'Ma Xiao' }, role: 'host', joined: '2026-10-02' },
    { id: 'li', name: { zh: '李步云', en: 'Li Buyun', de: 'Li Buyun' }, role: 'steward', joined: '2026-10-02' },
    { id: 'zhou', name: { zh: '周晓钟', en: 'Zhou Xiaozhong', de: 'Zhou Xiaozhong' }, role: 'guest', joined: '2026-10-02' },
    { id: 'wang', name: { zh: '王婧', en: 'Wang Jing', de: 'Wang Jing' }, role: 'guest', joined: '2026-10-03' },
    { id: 'chen', name: { zh: '陈默', en: 'Chen Mo', de: 'Chen Mo' }, role: 'guest', joined: '2026-10-03' },
    { id: 'seb', name: { zh: '塞巴斯蒂安', en: 'Sebastian', de: 'Sebastian' }, role: 'guest', joined: '2026-10-09' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '带塞巴斯蒂安逛一天：五个地方怎么走', en: 'One day with Sebastian: how to fit in five places', de: 'Ein Tag mit Sebastian: wie wir fünf Orte schaffen' },
      summary: {
        zh: '雷台、鸠摩罗什寺、大云寺、海藏寺、核桃园，只有一天。先去哪，在哪吃饭，哪里可以不去。',
        en: 'Leitai, Kumarajiva Temple, Dayun Temple, Haizang Temple, the Walnut Garden, and only one day. What first, where to eat, what to skip.',
        de: 'Leitai, Kumarajiva-Tempel, Dayun-Tempel, Haizang-Tempel, der Walnussgarten, und nur ein Tag. Was zuerst, wo essen, was weglassen.',
      },
      status: 'active',
      owner: 'li',
      by: 'ma',
      opened: '2026-10-03',
      updated: '2026-10-18T19:40',
      body: [
        {
          heading: { zh: '现状', en: 'Now', de: 'Stand' },
          text: {
            zh: '春节初三，塞巴斯蒂安和他的两个朋友从兰州坐车过来，晚上就走。我们五个都在，陈默有车。',
            en: 'On the third day of Spring Festival, Sebastian and two friends come over from Lanzhou and leave that evening. All five of us are home, and Chen Mo has a car.',
            de: 'Am dritten Tag des Frühlingsfests kommen Sebastian und zwei Freunde aus Lanzhou und fahren abends zurück. Wir fünf sind alle da, Chen Mo hat ein Auto.',
          },
        },
        {
          heading: { zh: '目标', en: 'Goal', de: 'Ziel' },
          text: {
            zh: '一张纸：先去哪、每处看什么、中午吃什么。让第一次来的人走完这一天，记得住凉州。',
            en: 'One sheet of paper: where to go first, what to see at each place, what to eat at noon. Someone here for the first time should leave remembering Liangzhou.',
            de: 'Ein Blatt Papier: wohin zuerst, was man an jedem Ort sieht, was es mittags gibt. Wer zum ersten Mal hier ist, soll sich an Liangzhou erinnern.',
          },
        },
        {
          heading: { zh: '约束', en: 'Limits', de: 'Rahmen' },
          text: {
            zh: '冬天冷，台上风大；一天最多走五处，走不完就删，不赶。',
            en: 'It’s cold in winter and windy up on the terrace; five places at most, and if it doesn’t fit we drop one rather than rush.',
            de: 'Im Winter ist es kalt und oben auf der Terrasse windig; höchstens fünf Orte, und wenn es nicht passt, lassen wir einen weg, statt zu hetzen.',
          },
        },
      ],
      works: [
        {
          id: 'x1', kind: 'plan', by: 'li', when: '2026-10-18T19:40', title: route,
          note: { zh: '第二稿：按上午、中午、下午排', en: 'Second draft: morning, noon, afternoon', de: 'Zweiter Entwurf: Vormittag, Mittag, Nachmittag' },
          lines: [
            { zh: '上午 · 雷台', en: 'Morning · Leitai', de: 'Vormittag · Leitai' },
            { zh: '看铜奔马出土的汉墓，再上台。抬头看牌匾：“步云”。', en: 'See the Han tomb where the bronze galloping horse was found, then climb the terrace. Look up at the plaque: “Bu Yun”, walking on clouds.', de: 'Das Han-Grab ansehen, in dem das galoppierende Bronzepferd gefunden wurde, dann auf die Terrasse. Oben die Tafel: „Bu Yun“, auf Wolken gehen.' },
            { zh: '中午 · 小北街', en: 'Noon · Xiaobei Street', de: 'Mittag · Xiaobei-Straße' },
            { zh: '面皮子、凉面。饭后一碗三炮台。', en: 'Mianpizi and cold noodles. A bowl of sanpaotai tea afterwards.', de: 'Mianpizi und kalte Nudeln. Danach eine Schale Sanpaotai-Tee.' },
            { zh: '下午 · 鸠摩罗什寺', en: 'Afternoon · Kumarajiva Temple', de: 'Nachmittag · Kumarajiva-Tempel' },
            { zh: '看罗什塔。出门时回头，看出口的牌匾：“人间净土”。', en: 'See the Kumarajiva pagoda. On the way out, turn round and read the plaque over the exit: “A Pure Land on Earth”.', de: 'Die Kumarajiva-Pagode ansehen. Beim Hinausgehen umdrehen und die Tafel über dem Ausgang lesen: „Reines Land auf Erden“.' },
            { zh: '下午 · 大云寺', en: 'Afternoon · Dayun Temple', de: 'Nachmittag · Dayun-Tempel' },
            { zh: '看钟楼和那口大钟。', en: 'See the bell tower and the great bell.', de: 'Den Glockenturm und die große Glocke ansehen.' },
            { zh: '天色还早 · 海藏寺', en: 'If there’s light left · Haizang Temple', de: 'Wenn es noch hell ist · Haizang-Tempel' },
            { zh: '核桃园留给下一次：冬天树都秃着，春天再来。', en: 'The Walnut Garden can wait: the trees are bare in winter. Come back in spring.', de: 'Der Walnussgarten muss warten: Im Winter sind die Bäume kahl. Im Frühling wiederkommen.' },
          ],
        },
        {
          id: 'x2', kind: 'table', by: 'zhou', when: '2026-10-16T21:15', title: plaques,
          note: { zh: '路过就抄一块，抄回来填上', en: 'Copy one down whenever you pass by, then fill it in', de: 'Wer vorbeikommt, schreibt eine ab und trägt sie ein' },
          lines: [
            { zh: '地方　牌匾　在哪　抄回来了吗', en: 'Place  Plaque  Where  Copied?', de: 'Ort  Tafel  Wo  Abgeschrieben?' },
            { zh: '雷台　步云　台上　抄了', en: 'Leitai  Bu Yun (walking on clouds)  on the terrace  yes', de: 'Leitai  Bu Yun (auf Wolken gehen)  auf der Terrasse  ja' },
            { zh: '鸠摩罗什寺　人间净土　出口　抄了', en: 'Kumarajiva Temple  A Pure Land on Earth  over the exit  yes', de: 'Kumarajiva-Tempel  Reines Land auf Erden  über dem Ausgang  ja' },
            { zh: '大云寺　—　—　还没有', en: 'Dayun Temple  —  —  not yet', de: 'Dayun-Tempel  —  —  noch nicht' },
            { zh: '海藏寺　—　—　还没有', en: 'Haizang Temple  —  —  not yet', de: 'Haizang-Tempel  —  —  noch nicht' },
            { zh: '核桃园　—　—　还没有', en: 'Walnut Garden  —  —  not yet', de: 'Walnussgarten  —  —  noch nicht' },
          ],
        },
        {
          id: 'x3', kind: 'doc', by: 'wang', when: '2026-10-12T22:30', title: forSeb,
          note: { zh: '用英文写，塞巴斯蒂安也能自己看', en: 'Written in English so Sebastian can read it himself', de: 'Auf Englisch, damit Sebastian es selbst lesen kann' },
          lines: [
            { zh: 'Wuwei was called Liangzhou, a city on the Hexi Corridor, the old road to the west.', en: 'Wuwei was called Liangzhou, a city on the Hexi Corridor, the old road to the west.', de: 'Wuwei was called Liangzhou, a city on the Hexi Corridor, the old road to the west.' },
            { zh: 'Kumarajiva lived here for many years before he went to Chang’an and became the great translator of Buddhist texts.', en: 'Kumarajiva lived here for many years before he went to Chang’an and became the great translator of Buddhist texts.', de: 'Kumarajiva lived here for many years before he went to Chang’an and became the great translator of Buddhist texts.' },
            { zh: 'The Western Xia stele has Tangut on one side and Chinese on the other; it helped people read Tangut again.', en: 'The Western Xia stele has Tangut on one side and Chinese on the other; it helped people read Tangut again.', de: 'The Western Xia stele has Tangut on one side and Chinese on the other; it helped people read Tangut again.' },
            { zh: 'The bronze galloping horse was found in the Han tomb at Leitai.', en: 'The bronze galloping horse was found in the Han tomb at Leitai.', de: 'The bronze galloping horse was found in the Han tomb at Leitai.' },
          ],
        },
        { id: 'x4', kind: 'image', by: 'chen', when: '2026-10-08T16:05', title: buyun },
      ],
      events: [
        { by: 'li', act: 'update', ref: route, when: '2026-10-18T19:40' },
        { by: 'zhou', act: 'propose', ref: morningBell, when: '2026-10-17T07:10' },
        { by: 'zhou', act: 'addWork', ref: plaques, when: '2026-10-16T21:15' },
        { by: 'wang', act: 'addWork', ref: forSeb, when: '2026-10-12T22:30' },
        { by: 'chen', act: 'addWork', ref: buyun, when: '2026-10-08T16:05' },
        { by: 'ma', act: 'markActive', when: '2026-10-04T09:00' },
        { by: 'ma', act: 'open', when: '2026-10-03T21:00' },
      ],
      notes: [
        { by: 'seb', when: '2026-10-12T23:05', text: { zh: '我最想看那块一面西夏文、一面汉文的碑。', en: 'What I most want to see is the stele with Tangut on one side and Chinese on the other.', de: 'Am meisten möchte ich die Stele sehen, mit Tangutisch auf der einen und Chinesisch auf der anderen Seite.' } },
        { by: 'ma', when: '2026-10-12T23:20', text: { zh: '西夏碑在博物馆里。我查一下初三开不开门，开就加一站。', en: 'The stele is in the museum. I’ll check whether it’s open on the third day; if it is, we add a stop.', de: 'Die Stele steht im Museum. Ich schaue, ob es am dritten Tag offen hat; wenn ja, kommt ein Halt dazu.' } },
        { by: 'zhou', when: '2026-10-17T07:15', text: { zh: '大云寺下午去，只能看钟，听不到。我交了个提案，挪到上午试试。', en: 'If we go to Dayun Temple in the afternoon we only see the bell, we don’t hear it. I’ve put in a proposal to move it to the morning.', de: 'Nachmittags sehen wir am Dayun-Tempel nur die Glocke, hören sie aber nicht. Ich habe vorgeschlagen, ihn auf den Morgen zu legen.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '面皮子哪家最地道', en: 'Whose mianpizi is the real thing?', de: 'Wo gibt es die echten Mianpizi?' },
      summary: {
        zh: '每个人心里都有一家。投个票，选两家带塞巴斯蒂安去，辣子另放。',
        en: 'Everyone has their own place. We vote, pick two to take Sebastian to, and ask for the chilli on the side.',
        de: 'Jeder hat seinen eigenen Laden. Wir stimmen ab, nehmen zwei für Sebastian und lassen das Chili extra geben.',
      },
      status: 'open', by: 'chen', opened: '2026-10-05', updated: '2026-10-15',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0003',
      title: { zh: '三炮台：冰糖放多少', en: 'Sanpaotai tea: how much rock sugar?', de: 'Sanpaotai-Tee: wie viel Kandiszucker?' },
      summary: {
        zh: '碗、盖、托三件。冰糖、桂圆、红枣、枸杞都放进去，一泡续好几次水。外地朋友第一口会不会太甜？',
        en: 'Bowl, lid and saucer. Rock sugar, longan, red dates and goji go in, and you top up the water again and again. Will the first sip be too sweet for visitors?',
        de: 'Schale, Deckel, Untertasse. Kandiszucker, Longan, rote Datteln und Goji kommen hinein, und man gießt immer wieder Wasser nach. Ist der erste Schluck für Gäste zu süß?',
      },
      status: 'open', by: 'wang', opened: '2026-10-10', updated: '2026-10-14',
      body: [], works: [], workCount: 0, events: [],
    },
    {
      id: '0004',
      title: { zh: '核桃园冬天开不开门', en: 'Is the Walnut Garden open in winter?', de: 'Hat der Walnussgarten im Winter offen?' },
      summary: {
        zh: '塞巴斯蒂安听说城里有个植物园，想去看看。冬天去值不值，开不开门，还没人问过。',
        en: 'Sebastian heard there’s a botanical garden in town and wants to go. Nobody has asked yet whether it’s open, or worth it, in winter.',
        de: 'Sebastian hat von einem botanischen Garten in der Stadt gehört und will hin. Ob er im Winter offen hat und sich lohnt, hat noch niemand gefragt.',
      },
      status: 'open', by: 'seb', opened: '2026-10-11', updated: '2026-10-11',
      body: [], works: [], workCount: 0, events: [],
    },
    {
      id: '0005',
      title: { zh: '凉面的醋：自己调还是买现成的', en: 'Vinegar for the cold noodles: mix our own or buy it ready?', de: 'Essig für die kalten Nudeln: selbst mischen oder fertig kaufen?' },
      summary: {
        zh: '定了：自己调，蒜水多放。周晓钟负责。',
        en: 'Decided: we mix our own, with plenty of garlic water. Zhou Xiaozhong is in charge.',
        de: 'Entschieden: selbst gemischt, mit viel Knoblauchwasser. Zhou Xiaozhong kümmert sich darum.',
      },
      status: 'solved', by: 'zhou', opened: '2026-10-06', updated: '2026-10-09',
      body: [], works: [], workCount: 1, events: [],
    },
  ],
  recent: [
    { by: 'li', problem: '0001', act: 'update', ref: route, when: '2026-10-18T19:40' },
    { by: 'zhou', problem: '0001', act: 'propose', ref: morningBell, when: '2026-10-17T07:10' },
    { by: 'zhou', problem: '0001', act: 'addWork', ref: plaques, when: '2026-10-16T21:15' },
    { by: 'chen', problem: '0002', act: 'addWork', ref: votes, when: '2026-10-15T12:30' },
  ],
  proposals: [
    {
      id: 'p1',
      title: morningBell,
      by: 'zhou',
      problem: '0001',
      when: '2026-10-17T07:10',
      reason: {
        zh: '大云晓钟，晓是早上。下午去只能看钟，听不到。雷台挪到下午，台上的“步云”下午看也一样。',
        en: '“Dayun Xiaozhong” means the dawn bell at Dayun. In the afternoon we would only see it, not hear it. Leitai can move to the afternoon; “Bu Yun” reads the same then.',
        de: '„Dayun Xiaozhong“ heißt die Morgenglocke von Dayun. Nachmittags würden wir sie nur sehen, nicht hören. Leitai kann auf den Nachmittag; „Bu Yun“ liest sich dann genauso.',
      },
      files: [
        {
          title: route,
          kind: 'plan',
          lines: [
            { kind: 'del', text: { zh: '上午 · 雷台', en: 'Morning · Leitai', de: 'Vormittag · Leitai' } },
            { kind: 'add', text: { zh: '上午 · 大云寺', en: 'Morning · Dayun Temple', de: 'Vormittag · Dayun-Tempel' } },
            { kind: 'add', text: { zh: '早点到，听晓钟。听不到也站在钟下看看。', en: 'Get there early for the dawn bell. If it doesn’t ring, stand under it anyway.', de: 'Früh da sein, wegen der Morgenglocke. Wenn sie nicht schlägt, trotzdem darunter stehen.' } },
            { kind: 'same', text: { zh: '中午 · 小北街', en: 'Noon · Xiaobei Street', de: 'Mittag · Xiaobei-Straße' } },
            { kind: 'same', text: { zh: '面皮子、凉面。饭后一碗三炮台。', en: 'Mianpizi and cold noodles. A bowl of sanpaotai tea afterwards.', de: 'Mianpizi und kalte Nudeln. Danach eine Schale Sanpaotai-Tee.' } },
            { kind: 'same', text: { zh: '下午 · 鸠摩罗什寺', en: 'Afternoon · Kumarajiva Temple', de: 'Nachmittag · Kumarajiva-Tempel' } },
            { kind: 'del', text: { zh: '下午 · 大云寺', en: 'Afternoon · Dayun Temple', de: 'Nachmittag · Dayun-Tempel' } },
            { kind: 'add', text: { zh: '下午 · 雷台，抬头看“步云”', en: 'Afternoon · Leitai, look up at “Bu Yun”', de: 'Nachmittag · Leitai, oben „Bu Yun“ lesen' } },
          ],
        },
      ],
    },
  ],
  decided: [
    { title: { zh: '凉面的醋自己调', en: 'We mix the noodle vinegar ourselves', de: 'Den Nudelessig mischen wir selbst' }, by: 'zhou', result: 'merged', when: '2026-10-09' },
  ],
  board: [
    {
      id: 'b1', by: 'ma', when: '2026-10-18T20:00',
      title: { zh: '初三早上九点，雷台门口集合', en: 'Third day, 9 am, meet at the Leitai gate', de: 'Dritter Tag, 9 Uhr, Treffpunkt am Tor von Leitai' },
      text: {
        zh: '按路线走，路线以 0001 的最新一稿为准。穿厚一点，台上风大。塞巴斯蒂安一行三个人坐陈默的车。',
        en: 'We follow the route; the latest draft under 0001 is the one that counts. Dress warmly, it’s windy on the terrace. Sebastian and his two friends ride with Chen Mo.',
        de: 'Wir folgen der Route; es gilt der neueste Entwurf unter 0001. Zieht euch warm an, oben auf der Terrasse ist es windig. Sebastian und seine zwei Freunde fahren bei Chen Mo mit.',
      },
      replies: [
        { by: 'seb', when: '2026-10-18T20:06', text: { zh: '收到！我们会准时到。', en: 'Got it! We’ll be on time.', de: 'Alles klar! Wir sind pünktlich.' } },
        { by: 'chen', when: '2026-10-18T20:08', text: { zh: '车能坐五个，八点半去火车站接他们。', en: 'The car seats five. I’ll pick them up at the station at half past eight.', de: 'Das Auto hat fünf Plätze. Ich hole sie um halb neun am Bahnhof ab.' } },
      ],
    },
    {
      id: 'b2', by: 'li', when: '2026-10-16T21:30',
      title: { zh: '牌匾还差三块', en: 'Three plaques still missing', de: 'Drei Tafeln fehlen noch' },
      text: {
        zh: '大云寺、海藏寺、核桃园的牌匾还没抄回来。谁路过拍一张，填进 0001 的《五处牌匾》。',
        en: 'We still haven’t copied the plaques at Dayun Temple, Haizang Temple and the Walnut Garden. Whoever passes by, take a photo and fill it into “Five plaques” under 0001.',
        de: 'Die Tafeln am Dayun-Tempel, am Haizang-Tempel und im Walnussgarten fehlen noch. Wer vorbeikommt, macht ein Foto und trägt sie in „Fünf Tafeln“ unter 0001 ein.',
      },
      replies: [],
    },
  ],
}
