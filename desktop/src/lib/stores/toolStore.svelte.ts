import type { ToolPanel } from "../types";

class ToolStore {
  activeTool = $state<ToolPanel>(null);

  // AI 聊天
  aiMessages = $state<{ role: string; content: string; time: number }[]>([
    { role: "ai", content: "你好！有什么可以帮助你的吗？", time: Date.now() - 300000 },
  ]);
  aiInput = $state("");

  // 快捷回复
  replyGroups = $state<{ id: string; name: string; items: { id: string; content: string }[] }[]>([
    {
      id: "g1", name: "产品介绍",
      items: [
        { id: "r1", content: "您好，我们是专业的外贸服务公司，专注于帮助中国企业拓展海外市场。" },
        { id: "r2", content: "我们的产品具有以下优势：品质保障、价格优惠、交期准时。" },
        { id: "r3", content: "我们提供免费样品，您满意后再下单。" },
      ],
    },
    {
      id: "g2", name: "常用话术",
      items: [
        { id: "r4", content: "感谢您的咨询！我会尽快为您处理。" },
        { id: "r5", content: "好的，我马上为您安排。" },
        { id: "r6", content: "请问您还有其他问题吗？" },
      ],
    },
    {
      id: "g3", name: "节日问候",
      items: [
        { id: "r7", content: "新年快乐！祝您生意兴隆！" },
        { id: "r8", content: "圣诞快乐！感谢您一年来的支持！" },
      ],
    },
  ]);

  // 翻译
  translateInput = $state("");
  translateResult = $state("");
  translateSourceLang = $state("auto");
  translateTargetLang = $state("zh-CN");
  translateHistory = $state<{ from: string; to: string; text: string; result: string; time: number }[]>([]);

  toggleTool(panel: ToolPanel) {
    this.activeTool = this.activeTool === panel ? null : panel;
  }

  closeTool() { this.activeTool = null; }

  sendAiMessage() {
    const text = this.aiInput.trim();
    if (!text) return;
    this.aiMessages = [...this.aiMessages, { role: "user", content: text, time: Date.now() }];
    this.aiInput = "";
    setTimeout(() => {
      this.aiMessages = [...this.aiMessages, {
        role: "ai",
        content: "好的，我了解了。让我为您处理这个问题。请稍候...",
        time: Date.now(),
      }];
    }, 800);
  }

  doTranslate() {
    const text = this.translateInput.trim();
    if (!text) return;
    const mockResult = `[${this.translateTargetLang.toUpperCase()}] ${text}`;
    this.translateResult = mockResult;
    this.translateHistory = [
      { from: this.translateSourceLang, to: this.translateTargetLang, text, result: mockResult, time: Date.now() },
      ...this.translateHistory,
    ];
  }
}

export const toolState = new ToolStore();
