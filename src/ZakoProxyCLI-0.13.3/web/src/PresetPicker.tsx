import { useEffect, useMemo, useRef, useState } from "react";
import { ClipboardPaste, ExternalLink, LoaderCircle, Search } from "lucide-react";
import { api, errorMessage } from "./api";
import type { ParsedCredential, ProviderPreset } from "./api";
import { SearchInput } from "./components";

import "./preset-picker.css";

const CATEGORY_LABEL: Record<ProviderPreset["category"], string> = {
  relay: "聚合中转",
  vendor: "官方直连",
  local: "本地与自建",
  custom: "自定义与导入",
};

const CATEGORY_ORDER: ProviderPreset["category"][] = ["vendor", "relay", "local", "custom"];

const AUTH_LABEL: Record<ProviderPreset["auth_style"], string> = {
  bearer: "Authorization: Bearer",
  anthropic: "x-api-key",
  google: "x-goog-api-key",
};

export type PresetSelection = {
  preset: ProviderPreset;
  endpoint: string;
  authStyle: ProviderPreset["auth_style"];
  protocols: string[];
  models: string[];
};

/**
 * Catalog picker for the "add account" dialog.
 *
 * Mirrors the add-account flow of Agent2API and magpie: choose a provider first, and the
 * dialog fills in the base URL, the credential header and a seed model list, so the
 * operator never has to recall which vendor wants `x-api-key` and which `/v1beta/openai`.
 * Everything it writes stays editable in the form below.
 */
export function PresetPicker({
  disabled,
  selected,
  onSelect,
  onPasteParse,
}: {
  disabled?: boolean;
  selected: string;
  onSelect: (selection: PresetSelection) => void;
  onPasteParse: (parsed: ParsedCredential) => void;
}) {
  const [presets, setPresets] = useState<ProviderPreset[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [pasteOpen, setPasteOpen] = useState(false);
  const [pasteText, setPasteText] = useState("");
  const [pasteBusy, setPasteBusy] = useState(false);
  const [pasteNote, setPasteNote] = useState("");
  const fetched = useRef(false);

  useEffect(() => {
    if (fetched.current) return;
    fetched.current = true;
    let live = true;
    api<{ presets: ProviderPreset[] }>("/presets")
      .then((result) => {
        if (live) setPresets(result.presets);
      })
      .catch((cause) => {
        if (live) setError(errorMessage(cause));
      })
      .finally(() => {
        if (live) setLoading(false);
      });
    return () => {
      live = false;
    };
  }, []);

  const groups = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const matched = presets.filter((preset) => {
      if (!needle) return true;
      return `${preset.name} ${preset.vendor} ${preset.id} ${preset.note}`
        .toLowerCase()
        .includes(needle);
    });
    return CATEGORY_ORDER.map((category) => ({
      category,
      items: matched.filter((preset) => preset.category === category),
    })).filter((group) => group.items.length > 0);
  }, [presets, query]);

  function choose(preset: ProviderPreset) {
    onSelect({
      preset,
      endpoint: preset.endpoint || "",
      authStyle: preset.auth_style,
      protocols: preset.protocols.length ? [...preset.protocols] : ["chat"],
      models: [...preset.models],
    });
    setError("");
  }

  async function parse() {
    const text = pasteText.trim();
    if (!text) {
      setPasteNote("先粘贴密钥、JSON 或客户端配置");
      return;
    }
    setPasteBusy(true);
    setPasteNote("");
    try {
      const parsed = await api<ParsedCredential>("/accounts/parse-credential", "POST", { text });
      if (parsed.source === "unrecognized") {
        setPasteNote("没能从这段内容里识别出密钥，请检查后重试");
      } else {
        const parts = [
          parsed.api_key ? "密钥已识别" : "未识别密钥",
          parsed.endpoint ? `端点 ${parsed.endpoint}` : "未识别端点",
        ];
        setPasteNote(parts.join(" · "));
        onPasteParse(parsed);
      }
    } catch (cause) {
      setPasteNote(errorMessage(cause));
    } finally {
      setPasteBusy(false);
    }
  }

  return (
    <div className="preset-picker">
      <div className="preset-picker-head">
        <span className="preset-picker-title">选择提供商</span>
        <button
          type="button"
          className="preset-paste-toggle"
          disabled={disabled}
          aria-expanded={pasteOpen}
          onClick={() => setPasteOpen((open) => !open)}
        >
          <ClipboardPaste size={14} />粘贴导入
        </button>
      </div>
      {pasteOpen ? (
        <div className="preset-paste">
          <textarea
            value={pasteText}
            onChange={(event) => setPasteText(event.target.value)}
            placeholder={'粘贴 API Key、{"api_key":…} 配置、OPENAI_API_KEY=… 环境变量，或一条 curl 命令'}
            rows={3}
            spellCheck={false}
            disabled={pasteBusy || disabled}
          />
          <div className="preset-paste-actions">
            <button type="button" disabled={pasteBusy || disabled} onClick={parse}>
              {pasteBusy ? <LoaderCircle size={14} className="spin" /> : null}识别并填入
            </button>
            {pasteNote ? <span className="preset-paste-note">{pasteNote}</span> : null}
          </div>
        </div>
      ) : null}
      <SearchInput value={query} onChange={setQuery} placeholder="搜索提供商，例如 deepseek、kimi、本地" />
      {loading ? (
        <p className="preset-hint"><LoaderCircle size={14} className="spin" />正在读取提供商目录…</p>
      ) : error ? (
        <p className="preset-hint preset-error">{error}</p>
      ) : groups.length === 0 ? (
        <p className="preset-hint">没有匹配的提供商，可以直接在下方填写端点。</p>
      ) : (
        <div className="preset-groups">
          {groups.map((group) => (
            <div className="preset-group" key={group.category}>
              <span className="preset-group-label">{CATEGORY_LABEL[group.category]}</span>
              <div className="preset-items">
                {group.items.map((preset) => (
                  <button
                    type="button"
                    key={preset.id}
                    className="preset-item"
                    aria-pressed={selected === preset.id}
                    disabled={disabled}
                    title={`${AUTH_LABEL[preset.auth_style]}${preset.endpoint ? ` · ${preset.endpoint}` : " · 需自行填写端点"}`}
                    onClick={() => choose(preset)}
                  >
                    <span className="preset-item-name">{preset.name}</span>
                    <span className="preset-item-note">{preset.note}</span>
                    {preset.docs ? (
                      <a
                        className="preset-item-docs"
                        href={preset.docs}
                        target="_blank"
                        rel="noreferrer noopener"
                        onClick={(event) => event.stopPropagation()}
                      >
                        <ExternalLink size={11} />
                      </a>
                    ) : null}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
      <p className="preset-hint">
        <Search size={13} />选中后会自动填入端点、认证方式与初始模型，仍可在下方修改。
      </p>
    </div>
  );
}
