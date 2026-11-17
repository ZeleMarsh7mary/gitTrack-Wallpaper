import { useCallback, useEffect, useRef, useState } from "react";
import { NavRail } from "./components/NavRail";
import { Preview } from "./components/Preview";
import { Toast } from "./components/Toast";
import { Button } from "./components/controls";
import { Icon } from "./components/Icons";
import { BackgroundPage } from "./pages/BackgroundPage";
import { MaterialPage } from "./pages/MaterialPage";
import { SourcePage } from "./pages/SourcePage";
import { TrackPage } from "./pages/TrackPage";
import * as ipc from "./lib/ipc";
import { errorText } from "./lib/ipc";
import type {
  AppStatus,
  PageId,
  ToastData,
  ToastKind,
  UiConfig,
  UpdateFn,
} from "./lib/types";

const PAGES: Record<PageId, { title: string; description: string }> = {
  source: {
    title: "Git source",
    description:
      "Pick the GitHub profile or repository that feeds the wallpaper.",
  },
  background: {
    title: "Background",
    description:
      "Choose an image, a flat colour or a gradient behind the card.",
  },
  material: {
    title: "Material & blur",
    description:
      "Tune the card's material, tint, blur and corner radius.",
  },
  track: {
    title: "Track & boxes",
    description:
      "Tune the heatmap squares, the colour ramp and Windows startup.",
  },
};

type Busy = "test" | "save" | "browse" | null;

export default function App() {
  const [page, setPage] = useState<PageId>("source");
  const [config, setConfig] = useState<UiConfig | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [rendering, setRendering] = useState(false);
  const [busy, setBusy] = useState<Busy>(null);
  const [footer, setFooter] = useState("Ready");
  const [connection, setConnection] = useState<number | null>(null);
  const [toast, setToast] = useState<ToastData | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  const configRef = useRef<UiConfig | null>(null);
  const renderSeq = useRef(0);
  const firstConfig = useRef(true);
  const toastSeq = useRef(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  const notify = useCallback((kind: ToastKind, text: string) => {
    toastSeq.current += 1;
    setToast({ id: toastSeq.current, kind, text });
    setFooter(text);
  }, []);

  const renderPreview = useCallback(
    async (next: UiConfig) => {
      renderSeq.current += 1;
      const seq = renderSeq.current;
      setRendering(true);
      try {
        const url = await ipc.renderPreview(next);
        if (seq !== renderSeq.current) return;
        setPreview(url);
        setPreviewError(null);
      } catch (error) {
        if (seq !== renderSeq.current) return;
        const message = errorText(error);
        setPreviewError(message);
        notify("error", message);
      } finally {
        if (seq === renderSeq.current) setRendering(false);
      }
    },
    [notify],
  );

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoadError(null);
      try {
        const boot = await ipc.bootstrap();
        if (cancelled) return;
        firstConfig.current = true;
        configRef.current = boot.config;
        setConfig(boot.config);
        setStatus(boot.status);
        setFooter("Ready");

        try {
          const micaApplied = await ipc.showWindow();
          if (!micaApplied) document.documentElement.classList.add("no-mica");
        } catch {
          document.documentElement.classList.add("no-mica");
        }

        if (!cancelled) void renderPreview(boot.config);
      } catch (error) {
        if (!cancelled) setLoadError(errorText(error));
      }
    };

    void load();
    return () => {
      cancelled = true;
    };
  }, [reloadKey, renderPreview]);

  useEffect(() => {
    if (!config) return;
    if (firstConfig.current) {
      firstConfig.current = false;
      return;
    }
    const timer = window.setTimeout(() => {
      void renderPreview(config);
    }, 150);
    return () => window.clearTimeout(timer);
  }, [config, renderPreview]);

  useEffect(() => {
    if (!toast) return;
    const id = window.setTimeout(() => {
      setToast((current) => (current && current.id === toast.id ? null : current));
    }, 4000);
    return () => window.clearTimeout(id);
  }, [toast]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: 0 });
  }, [page]);

  const update: UpdateFn = useCallback((recipe) => {
    setConfig((current) => {
      if (!current) return current;
      const next = recipe(current);
      configRef.current = next;
      return next;
    });
  }, []);

  const onBrowse = async () => {
    if (busy) return;
    setBusy("browse");
    try {
      const path = await ipc.pickImageFile();
      if (path) {
        update((current) => ({
          ...current,
          background: { ...current.background, imagePath: path, mode: "image" },
        }));
        setFooter("Background image selected.");
      }
    } catch (error) {
      notify("error", errorText(error));
    } finally {
      setBusy(null);
    }
  };

  const onTest = async () => {
    const current = configRef.current;
    if (!current || busy) return;
    setBusy("test");
    try {
      const stats = await ipc.testConnection(current);
      setConnection(stats.total);
      notify(
        "success",
        `Connection OK: ${stats.total.toLocaleString("en-US")} contributions in the last year.`,
      );
      void renderPreview(current);
    } catch (error) {
      notify("error", errorText(error));
    } finally {
      setBusy(null);
    }
  };

  const onSave = async () => {
    const current = configRef.current;
    if (!current || busy) return;
    setBusy("save");
    try {
      const result = await ipc.saveAndApply(current);
      setStatus((previous) =>
        previous
          ? {
              ...previous,
              daemonRunning: result.daemonRunning,
              autostart: result.autostart,
            }
          : previous,
      );
      notify("success", result.message);
    } catch (error) {
      notify("error", errorText(error));
    } finally {
      setBusy(null);
    }
  };

  const actionsDisabled = busy !== null;
  const meta = PAGES[page];

  const pageBody =
    config === null ? null : page === "source" ? (
      <SourcePage config={config} update={update} />
    ) : page === "background" ? (
      <BackgroundPage
        config={config}
        update={update}
        onBrowse={() => void onBrowse()}
        browseBusy={busy === "browse"}
        actionsDisabled={actionsDisabled}
      />
    ) : page === "material" ? (
      <MaterialPage config={config} update={update} />
    ) : (
      <TrackPage config={config} update={update} />
    );

  return (
    <div className="app">
      <NavRail page={page} onSelect={setPage} status={status} />

      <main className="main">
        {loadError ? (
          <div className="state-panel" role="alert">
            <span className="state-icon state-icon-error">
              <Icon name="warningCircle" size={22} />
            </span>
            <h1 className="state-title">Couldn’t load settings</h1>
            <p className="state-text">{loadError}</p>
            <Button variant="secondary" onClick={() => setReloadKey((key) => key + 1)}>
              Try again
            </Button>
          </div>
        ) : !config || !status ? (
          <div className="state-panel" aria-live="polite">
            <span className="spinner spinner-lg" aria-hidden="true" />
            <p className="state-text">Loading settings…</p>
          </div>
        ) : (
          <>
            <div className="scroll" ref={scrollRef}>
              <div className="page" key={page}>
                <header className="page-head">
                  <div className="page-head-text">
                    <h1 className="page-title">{meta.title}</h1>
                    <p className="page-desc">{meta.description}</p>
                  </div>
                  {page === "source" && connection !== null ? (
                    <span className="status-chip">
                      <span className="status-chip-dot" />
                      Connected: {connection.toLocaleString("en-US")} contributions
                    </span>
                  ) : null}
                </header>

                <Preview
                  src={preview}
                  updating={rendering}
                  error={previewError}
                  resolution={status.resolution}
                />

                {pageBody}
              </div>
            </div>

            <footer className="footer">
              <p className="footer-status" title={footer}>
                {footer}
              </p>
              <div className="footer-actions">
                <Button
                  variant="secondary"
                  onClick={() => void onTest()}
                  busy={busy === "test"}
                  busyLabel="Testing…"
                  disabled={actionsDisabled}
                >
                  Test Connection
                </Button>
                <Button
                  variant="primary"
                  onClick={() => void onSave()}
                  busy={busy === "save"}
                  busyLabel="Saving…"
                  disabled={actionsDisabled}
                >
                  Save &amp; Apply
                </Button>
              </div>
            </footer>
          </>
        )}

        <Toast toast={toast} />
      </main>
    </div>
  );
}
