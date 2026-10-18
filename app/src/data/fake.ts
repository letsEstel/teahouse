// M0 假数据：梧桐里三号院，一间解决院子里琐事的茶馆。

export type Role = 'boss' | 'heir' | 'guest'
export type Status = 'open' | 'active' | 'solved'
export type WorkKind = 'doc' | 'bill' | 'plan' | 'image'

export interface Member { id: string; name: string; role: Role; joined: string }
export interface Work { id: string; kind: WorkKind; title: string; by: string; when: string; note?: string }
export interface Event { by: string; text: string; when: string }
export interface Problem {
  id: string
  title: string
  summary: string
  status: Status
  by: string
  opened: string
  updated: string
  body: { heading: string; text: string }[]
  works: Work[]
  events: Event[]
}
export interface DiffLine { kind: 'same' | 'add' | 'del'; text: string }
export interface Proposal {
  id: string
  title: string
  by: string
  problem: string
  when: string
  reason: string
  files: { title: string; kind: WorkKind; lines: DiffLine[] }[]
}

export const house = {
  name: '梧桐里三号院',
  motto: '院子里的事，有人办成。',
  founded: '2026.03.14',
  capacity: 15,
}

export const me = 'chen'

export const members: Member[] = [
  { id: 'chen', name: '陈立', role: 'boss', joined: '03.14' },
  { id: 'zhou', name: '周小满', role: 'heir', joined: '03.14' },
  { id: 'wang', name: '王阿姨', role: 'guest', joined: '03.16' },
  { id: 'li', name: '李建国', role: 'guest', joined: '03.16' },
  { id: 'zhao', name: '赵一鸣', role: 'guest', joined: '03.22' },
  { id: 'sun', name: '孙婷', role: 'guest', joined: '04.02' },
  { id: 'wu', name: '吴师傅', role: 'guest', joined: '04.09' },
  { id: 'zheng', name: '郑可', role: 'guest', joined: '05.30' },
  { id: 'lin', name: '林晚', role: 'guest', joined: '09.21' },
]

export const roleName: Record<Role, string> = { boss: '老板', heir: '伙计', guest: '茶客' }
export const statusName: Record<Status, string> = { open: '待认领', active: '进行中', solved: '已办成' }
export const kindName: Record<WorkKind, string> = { doc: '文档', bill: '账单', plan: '方案', image: '图片' }

export const byId = (id: string) => members.find((m) => m.id === id)!

export const problems: Problem[] = [
  {
    id: '0001',
    title: '小区停车位不好找',
    summary: '晚上七点以后院内车位全满，外来车占位，住户只能停到两条街外。',
    status: 'active',
    by: 'wang',
    opened: '03.16',
    updated: '2 小时前',
    body: [
      { heading: '现状', text: '院内共 42 个车位，登记住户车辆 57 辆。晚高峰（19:00 以后）车位全满，其中约 8 个被外来车辆长期占用。' },
      { heading: '目标', text: '年底前让每户至少有一个可预期的停车位置；外来车辆占位降到 2 个以下。' },
      { heading: '约束', text: '不新增地面硬化；物业不额外收费；方案需要业委会表决通过。' },
    ],
    works: [
      { id: 'w1', kind: 'image', title: '车位分布图（实测）', by: 'zhou', when: '2 小时前', note: '42 个车位逐一标号' },
      { id: 'w2', kind: 'plan', title: '错峰共享方案 v2', by: 'zhao', when: '昨天', note: '白天空位开放给夜班住户' },
      { id: 'w3', kind: 'bill', title: '地锁采购报价', by: 'li', when: '10.12', note: '三家对比，含安装' },
      { id: 'w4', kind: 'doc', title: '一周占位记录', by: 'wang', when: '10.09', note: '每晚 21:00 拍照统计' },
      { id: 'w5', kind: 'doc', title: '致物业的函（草稿）', by: 'chen', when: '10.03' },
      { id: 'w6', kind: 'bill', title: '周边收费停车场价目', by: 'sun', when: '09.27' },
      { id: 'w7', kind: 'image', title: '外来车辆照片汇总', by: 'wu', when: '09.20' },
      { id: 'w8', kind: 'plan', title: '错峰共享方案 v1', by: 'zhao', when: '09.18' },
      { id: 'w9', kind: 'doc', title: '住户问卷结果', by: 'zheng', when: '09.11' },
    ],
    events: [
      { by: 'zhou', text: '添加了成果《车位分布图（实测）》', when: '2 小时前' },
      { by: 'zhao', text: '发起提案《把夜间临时停车改为错峰共享》', when: '昨天 22:14' },
      { by: 'chen', text: '合并了提案《地锁采购报价》', when: '10.12' },
      { by: 'wang', text: '更新了《一周占位记录》', when: '10.09' },
      { by: 'chen', text: '将问题标为进行中', when: '09.18' },
      { by: 'wang', text: '提出了这个问题', when: '03.16' },
    ],
  },
  {
    id: '0002',
    title: '垃圾分类点的气味',
    summary: '夏季分类点紧挨 2 号楼窗下，异味严重，需要挪位置或加清运频次。',
    status: 'active',
    by: 'sun',
    opened: '06.02',
    updated: '昨天',
    body: [], works: [
      { id: 'a', kind: 'doc', title: '', by: 'sun', when: '' }, { id: 'b', kind: 'doc', title: '', by: 'sun', when: '' },
      { id: 'c', kind: 'doc', title: '', by: 'sun', when: '' }, { id: 'd', kind: 'doc', title: '', by: 'sun', when: '' },
      { id: 'e', kind: 'doc', title: '', by: 'sun', when: '' },
    ], events: [],
  },
  {
    id: '0003',
    title: '楼道灯坏了三个月',
    summary: '3 号楼 4–6 层声控灯失灵。已找到电工，费用由维修基金支出。',
    status: 'solved',
    by: 'wu',
    opened: '04.10',
    updated: '07.21',
    body: [], works: [
      { id: 'a', kind: 'doc', title: '', by: 'wu', when: '' }, { id: 'b', kind: 'doc', title: '', by: 'wu', when: '' },
      { id: 'c', kind: 'doc', title: '', by: 'wu', when: '' }, { id: 'd', kind: 'doc', title: '', by: 'wu', when: '' },
    ], events: [],
  },
  {
    id: '0004',
    title: '给院里独居老人订午餐',
    summary: '院内 6 位独居老人，想联系附近食堂按周订餐、轮流送到门口。',
    status: 'open',
    by: 'lin',
    opened: '10.15',
    updated: '10.15',
    body: [], works: [
      { id: 'a', kind: 'doc', title: '', by: 'lin', when: '' }, { id: 'b', kind: 'doc', title: '', by: 'lin', when: '' },
    ], events: [],
  },
  {
    id: '0005',
    title: '年底公共维修基金怎么花',
    summary: '账上余 3.2 万元，需要在 12 月业委会前列出优先级清单。',
    status: 'open',
    by: 'chen',
    opened: '10.08',
    updated: '10.14',
    body: [], works: [
      { id: 'a', kind: 'bill', title: '', by: 'chen', when: '' }, { id: 'b', kind: 'bill', title: '', by: 'chen', when: '' },
      { id: 'c', kind: 'bill', title: '', by: 'chen', when: '' },
    ], events: [],
  },
]

export const recent: (Event & { problem: string })[] = [
  { by: 'zhou', problem: '0001', text: '添加了成果《车位分布图（实测）》', when: '2 小时前' },
  { by: 'zhao', problem: '0001', text: '发起提案《把夜间临时停车改为错峰共享》', when: '昨天' },
  { by: 'sun', problem: '0002', text: '添加了成果《清运时间表》', when: '昨天' },
  { by: 'lin', problem: '0004', text: '提出了问题《给院里独居老人订午餐》', when: '10.15' },
  { by: 'chen', problem: '0005', text: '添加了成果《维修基金余额明细》', when: '10.14' },
]

export const proposals: Proposal[] = [
  {
    id: 'p1',
    title: '把夜间临时停车改为错峰共享',
    by: 'zhao',
    problem: '0001',
    when: '昨天 22:14',
    reason: 'v1 只解决了夜间外来车，没有考虑夜班住户。问卷里有 11 户上夜班，白天车位空着、晚上回来没地方。改成按时段共享，不需要装地锁。',
    files: [
      {
        title: '错峰共享方案',
        kind: 'plan',
        lines: [
          { kind: 'same', text: '一、适用范围' },
          { kind: 'same', text: '院内全部 42 个地面车位。' },
          { kind: 'same', text: '' },
          { kind: 'same', text: '二、时段划分' },
          { kind: 'del', text: '19:00–07:00 禁止外来车辆进入，由门卫登记放行。' },
          { kind: 'add', text: '07:00–19:00 为日间时段：登记的夜班住户可停任意空位。' },
          { kind: 'add', text: '19:00–07:00 为夜间时段：车位优先归属固定住户，夜班住户须在 19:30 前驶离。' },
          { kind: 'same', text: '' },
          { kind: 'same', text: '三、执行方式' },
          { kind: 'del', text: '在 8 个高频被占车位安装地锁，钥匙由住户自管。' },
          { kind: 'add', text: '不安装地锁。每个车位喷涂编号，在院门口张贴当日共享表。' },
          { kind: 'add', text: '违规三次的车辆，由伙计统一联系车主。' },
          { kind: 'same', text: '外来车辆仍需门卫登记。' },
        ],
      },
      {
        title: '地锁采购报价',
        kind: 'bill',
        lines: [
          { kind: 'same', text: '供应商　　　单价　　数量　　小计' },
          { kind: 'del', text: '安途五金　　¥186　　8　　　¥1,488' },
          { kind: 'add', text: '（方案 v2 不再采购地锁，本账单作废留档）' },
        ],
      },
    ],
  },
  {
    id: 'p2',
    title: '补充 2 号楼窗下实测气味记录',
    by: 'sun',
    problem: '0002',
    when: '10.16',
    reason: '',
    files: [],
  },
]
