// 小北街：核桃园里的第七个茶摊子（docs/NAMES.zh.md）。春节，五个外国朋友（德国、美国、缅甸、阿根廷、
// 新西兰）来凉州，六个本地人带路：一天、五个地方、几样吃的，一张纸写清楚。当前用户是老板（马晓）。
// 《五处牌匾》只填已知的两块，其余写“还没抄回来”，不编。核桃园没有门，也就没有门匾。
import type { House } from './types'

const route = { zh: '凉州一日路线', en: 'One day in Liangzhou', de: 'Ein Tag in Liangzhou' }
const plaques = { zh: '五处牌匾', en: 'Five plaques', de: 'Fünf Tafeln' }
const forGuests = { zh: '给五位朋友的一页介绍', en: 'One page for our five guests', de: 'Eine Seite für unsere fünf Gäste' }
const buyun = { zh: '雷台的“步云”', en: 'Leitai: “Bu Yun”', de: 'Leitai: „Bu Yun“' }
const morningBell = { zh: '路线：大云寺挪到上午，去听晓钟', en: 'Route: move Dayun Temple to the morning, for the dawn bell', de: 'Route: Dayun-Tempel auf den Morgen legen, wegen der Morgenglocke' }
const votes = { zh: '面皮子投票', en: 'Mianpizi vote', de: 'Abstimmung: Mianpizi' }

export const xiaobei: House = {
  id: 'xiaobei',
  name: { zh: '小北街', en: 'Xiaobei Street', de: 'Xiaobei-Straße' },
  nameLines: { zh: ['小北街'], en: ['Xiaobei', 'Street'], de: ['Xiaobei-', 'Straße'] },
  sign: {
    zh: '春节，五个外国朋友来凉州，六个本地人带路。一天，五个地方，几样吃的，一张纸写清楚。',
    en: 'Spring Festival: five friends from abroad come to Liangzhou, six locals show them round. One day, five places, a few things to eat, all on one sheet of paper.',
    de: 'Frühlingsfest: Fünf Freunde aus dem Ausland kommen nach Liangzhou, sechs Einheimische führen sie herum. Ein Tag, fünf Orte, ein paar Gerichte, alles auf einem Blatt.',
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
    { id: 'zhao', name: { zh: '赵海宁', en: 'Zhao Haining', de: 'Zhao Haining' }, role: 'guest', joined: '2026-10-04' },
    { id: 'seb', name: { zh: 'Sebastian', en: 'Sebastian', de: 'Sebastian' }, role: 'guest', joined: '2026-10-09' },
    { id: 'mike', name: { zh: 'Michael', en: 'Michael', de: 'Michael' }, role: 'guest', joined: '2026-10-09' },
    { id: 'aung', name: { zh: 'Aung Ko', en: 'Aung Ko', de: 'Aung Ko' }, role: 'guest', joined: '2026-10-10' },
    { id: 'lucia', name: { zh: 'Lucía', en: 'Lucía', de: 'Lucía' }, role: 'guest', joined: '2026-10-10' },
    { id: 'liam', name: { zh: 'Liam', en: 'Liam', de: 'Liam' }, role: 'guest', joined: '2026-10-11' },
  ],
  problems: [
    {
      id: '0001',
      title: { zh: '一天逛五个地方：怎么走', en: 'Five places in one day: in what order', de: 'Fünf Orte an einem Tag: in welcher Reihenfolge' },
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
            zh: '春节初三，五个朋友从德国、美国、缅甸、阿根廷、新西兰来，在兰州会合，坐早上的火车过来，晚上就走。我们六个本地人都在，陈默和赵海宁有车。',
            en: 'On the third day of Spring Festival, five friends from Germany, the US, Myanmar, Argentina and New Zealand meet in Lanzhou, take the morning train over and leave that evening. All six of us locals are home; Chen Mo and Zhao Haining have cars.',
            de: 'Am dritten Tag des Frühlingsfests treffen sich fünf Freunde aus Deutschland, den USA, Myanmar, Argentinien und Neuseeland in Lanzhou, nehmen den Morgenzug und fahren abends zurück. Wir sechs Einheimischen sind alle da; Chen Mo und Zhao Haining haben ein Auto.',
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
            { zh: '核桃园　没有门，也就没有门匾　—　不用抄', en: 'Walnut Garden  no gate, so no gate plaque  —  nothing to copy', de: 'Walnussgarten  kein Tor, also keine Tortafel  —  nichts abzuschreiben' },
          ],
        },
        {
          id: 'x3', kind: 'doc', by: 'wang', when: '2026-10-12T22:30', title: forGuests,
          note: { zh: '用英文写，五个人都能自己看', en: 'Written in English so all five can read it themselves', de: 'Auf Englisch, damit alle fünf es selbst lesen können' },
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
        { by: 'wang', act: 'addWork', ref: forGuests, when: '2026-10-12T22:30' },
        { by: 'chen', act: 'addWork', ref: buyun, when: '2026-10-08T16:05' },
        { by: 'ma', act: 'markActive', when: '2026-10-04T09:00' },
        { by: 'ma', act: 'open', when: '2026-10-03T21:00' },
      ],
      notes: [
        { by: 'seb', when: '2026-10-12T23:05', text: { zh: '我最想看那块一面西夏文、一面汉文的碑。', en: 'What I most want to see is the stele with Tangut on one side and Chinese on the other.', de: 'Am meisten möchte ich die Stele sehen, mit Tangutisch auf der einen und Chinesisch auf der anderen Seite.' } },
        { by: 'ma', when: '2026-10-12T23:20', text: { zh: '西夏碑在博物馆里。我查一下初三开不开门，开就加一站。', en: 'The stele is in the museum. I’ll check whether it’s open on the third day; if it is, we add a stop.', de: 'Die Stele steht im Museum. Ich schaue, ob es am dritten Tag offen hat; wenn ja, kommt ein Halt dazu.' } },
        { by: 'aung', when: '2026-10-13T08:40', text: { zh: '罗什寺我想多待一会儿。在缅甸，我每个星期都去寺里，想看看这里的寺是什么样子。', en: 'I’d like to stay a little longer at the Kumarajiva Temple. At home in Myanmar I go to the temple every week; I want to see what one looks like here.', de: 'Im Kumarajiva-Tempel würde ich gern etwas länger bleiben. Zu Hause in Myanmar gehe ich jede Woche in den Tempel; ich möchte sehen, wie einer hier aussieht.' } },
        { by: 'zhou', when: '2026-10-17T07:15', text: { zh: '大云寺下午去，只能看钟，听不到。我交了个提案，挪到上午试试。', en: 'If we go to Dayun Temple in the afternoon we only see the bell, we don’t hear it. I’ve put in a proposal to move it to the morning.', de: 'Nachmittags sehen wir am Dayun-Tempel nur die Glocke, hören sie aber nicht. Ich habe vorgeschlagen, ihn auf den Morgen zu legen.' } },
      ],
    },
    {
      id: '0002',
      title: { zh: '面皮子哪家最地道', en: 'Whose mianpizi is the real thing?', de: 'Wo gibt es die echten Mianpizi?' },
      summary: {
        zh: '每个人心里都有一家。投个票，选两家带朋友们去，辣子另放。',
        en: 'Everyone has their own place. We vote, pick two to take our friends to, and ask for the chilli on the side.',
        de: 'Jeder hat seinen eigenen Laden. Wir stimmen ab, nehmen zwei für unsere Freunde und lassen das Chili extra geben.',
      },
      status: 'open', by: 'chen', opened: '2026-10-05', updated: '2026-10-15',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0003',
      title: { zh: '三炮台去哪里喝', en: 'Where do we drink sanpaotai tea?', de: 'Wo trinken wir Sanpaotai-Tee?' },
      summary: {
        zh: '盖碗茶，碗、盖、托三件，要坐下来慢慢喝。找一家能坐下十一个人、下午不赶人的茶屋。',
        en: 'Tea in a lidded bowl, three pieces: bowl, lid, saucer. You have to sit down and take your time. We need a tea house with room for eleven that won’t rush us in the afternoon.',
        de: 'Tee in der Deckelschale, drei Teile: Schale, Deckel, Untertasse. Man muss sich setzen und Zeit lassen. Wir brauchen eine Teestube mit Platz für elf, die uns am Nachmittag nicht hinausdrängt.',
      },
      status: 'active', owner: 'wang', by: 'wang', opened: '2026-10-10', updated: '2026-10-16',
      body: [], works: [], workCount: 1, events: [],
    },
    {
      id: '0004',
      title: { zh: '大盘鸡去哪里吃', en: 'Where do we eat dapanji?', de: 'Wo essen wir Dapanji?' },
      summary: {
        zh: '晚饭吃大盘鸡，最后下皮带面。十一个人要两大盘。Lucía 和 Liam 不太能吃辣，一盘少放辣子。',
        en: 'Dapanji for dinner, with the wide belt noodles at the end. Eleven people need two big plates. Lucía and Liam can’t take much chilli, so one plate goes easy on it.',
        de: 'Zum Abendessen Dapanji, zum Schluss die breiten Gürtelnudeln. Elf Leute brauchen zwei große Platten. Lucía und Liam vertragen wenig Chili, also eine Platte mild.',
      },
      status: 'open', by: 'zhao', opened: '2026-10-12', updated: '2026-10-17',
      body: [], works: [], workCount: 0, events: [],
    },
    {
      id: '0005',
      title: { zh: '从兰州怎么过来：火车还是包车', en: 'Getting here from Lanzhou: train or a hired car?', de: 'Von Lanzhou hierher: Zug oder Mietwagen?' },
      summary: {
        zh: '定了：坐早上的火车，五个人一起走；陈默和赵海宁开车去车站接。',
        en: 'Decided: the morning train, all five together; Chen Mo and Zhao Haining drive to the station to pick them up.',
        de: 'Entschieden: der Morgenzug, alle fünf zusammen; Chen Mo und Zhao Haining holen sie mit dem Auto am Bahnhof ab.',
      },
      status: 'solved', by: 'mike', opened: '2026-10-09', updated: '2026-10-11',
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
    { title: { zh: '坐早上的火车过来', en: 'Coming by the morning train', de: 'Anreise mit dem Morgenzug' }, by: 'mike', result: 'merged', when: '2026-10-11' },
  ],
  board: [
    {
      id: 'b1', by: 'ma', when: '2026-10-18T20:00',
      title: { zh: '初三早上九点，雷台门口集合', en: 'Third day, 9 am, meet at the Leitai gate', de: 'Dritter Tag, 9 Uhr, Treffpunkt am Tor von Leitai' },
      text: {
        zh: '按路线走，路线以 0001 的最新一稿为准。穿厚一点，台上风大。五个朋友坐陈默和赵海宁的车。',
        en: 'We follow the route; the latest draft under 0001 is the one that counts. Dress warmly, it’s windy on the terrace. Our five friends ride with Chen Mo and Zhao Haining.',
        de: 'Wir folgen der Route; es gilt der neueste Entwurf unter 0001. Zieht euch warm an, oben auf der Terrasse ist es windig. Unsere fünf Freunde fahren bei Chen Mo und Zhao Haining mit.',
      },
      replies: [
        { by: 'seb', when: '2026-10-18T20:06', text: { zh: '收到！我们会准时到。', en: 'Got it! We’ll be on time.', de: 'Alles klar! Wir sind pünktlich.' } },
        { by: 'liam', when: '2026-10-18T20:07', text: { zh: '新西兰现在是夏天。我得借一件厚外套。', en: 'It’s summer in New Zealand right now. I’ll have to borrow a thick coat.', de: 'In Neuseeland ist gerade Sommer. Ich muss mir einen dicken Mantel leihen.' } },
        { by: 'chen', when: '2026-10-18T20:08', text: { zh: '两辆车，八点半去火车站接。外套我家有。', en: 'Two cars, at the station by half past eight. I have a coat you can borrow.', de: 'Zwei Autos, um halb neun am Bahnhof. Einen Mantel habe ich für dich.' } },
      ],
    },
    {
      id: 'b2', by: 'li', when: '2026-10-16T21:30',
      title: { zh: '牌匾还差两块', en: 'Two plaques still missing', de: 'Zwei Tafeln fehlen noch' },
      text: {
        zh: '大云寺、海藏寺的牌匾还没抄回来。谁路过拍一张，填进 0001 的《五处牌匾》。核桃园没有门，不用找了。',
        en: 'We still haven’t copied the plaques at Dayun Temple and Haizang Temple. Whoever passes by, take a photo and fill it into “Five plaques” under 0001. The Walnut Garden has no gate, so don’t look for one there.',
        de: 'Die Tafeln am Dayun-Tempel und am Haizang-Tempel fehlen noch. Wer vorbeikommt, macht ein Foto und trägt sie in „Fünf Tafeln“ unter 0001 ein. Der Walnussgarten hat kein Tor, dort braucht ihr nicht zu suchen.',
      },
      replies: [],
    },
  ],
}
