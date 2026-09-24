import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell } from "../../shared/AppShell";
import { isTauriAvailable } from "../../shared/tauri";
import "./CalendarPage.css";

type CalendarView = "day" | "week" | "month";
type CalendarEntry = { id: string; projectId: string; entryDate: string; content: string; createdAt: number; updatedAt: number };

const weekday = new Intl.DateTimeFormat("pt-BR", { weekday: "short" });
const dayLabel = new Intl.DateTimeFormat("pt-BR", { weekday: "long", day: "numeric", month: "long" });
const monthLabel = new Intl.DateTimeFormat("pt-BR", { month: "long", year: "numeric" });

function dateKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function parseDate(value: string) {
  const [year, month, day] = value.split("-").map(Number);
  return new Date(year, month - 1, day, 12);
}

function addDays(date: Date, days: number) {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

function weekStart(date: Date) {
  const offset = (date.getDay() + 6) % 7;
  return addDays(date, -offset);
}

function datesBetween(start: Date, end: Date) {
  const dates: Date[] = [];
  for (let current = new Date(start); current <= end; current = addDays(current, 1)) dates.push(new Date(current));
  return dates;
}

function visibleRange(selected: Date, view: CalendarView) {
  if (view === "day") return { start: selected, end: selected };
  if (view === "week") {
    const start = weekStart(selected);
    return { start, end: addDays(start, 6) };
  }
  const first = new Date(selected.getFullYear(), selected.getMonth(), 1, 12);
  const last = new Date(selected.getFullYear(), selected.getMonth() + 1, 0, 12);
  return { start: first, end: last };
}

function CalendarEntries({ entries, selectedDate, onSelectDate }: { entries: CalendarEntry[]; selectedDate: string; onSelectDate: (value: string) => void }) {
  if (!entries.length) return <p className="calendar-empty">Nenhum planejamento neste período.</p>;
  return <div className="calendar-entry-list">{entries.map((entry) => <button type="button" key={entry.id} className={entry.entryDate === selectedDate ? "selected" : ""} onClick={() => onSelectDate(entry.entryDate)}><time>{entry.entryDate === selectedDate ? "Dia selecionado" : parseDate(entry.entryDate).toLocaleDateString("pt-BR", { day: "2-digit", month: "short" })}</time><span>{entry.content}</span></button>)}</div>;
}

export default function CalendarPage({ projectId }: { projectId: string }) {
  const [view, setView] = useState<CalendarView>("month");
  const [selectedDate, setSelectedDate] = useState(() => dateKey(new Date()));
  const [entries, setEntries] = useState<CalendarEntry[]>([]);
  const [draft, setDraft] = useState("");
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState("");
  const selected = useMemo(() => parseDate(selectedDate), [selectedDate]);
  const range = useMemo(() => visibleRange(selected, view), [selected, view]);
  const periodDays = useMemo(() => datesBetween(range.start, range.end), [range]);
  const entriesByDate = useMemo(() => entries.reduce<Record<string, CalendarEntry[]>>((result, entry) => ({ ...result, [entry.entryDate]: [...(result[entry.entryDate] ?? []), entry] }), {}), [entries]);

  useEffect(() => {
    if (!isTauriAvailable()) {
      setEntries([]);
      setLoading(false);
      setMessage("Abra o Orbit pelo aplicativo desktop para usar o Calendar local.");
      return;
    }
    setLoading(true);
    setMessage("");
    void invoke<CalendarEntry[]>("list_calendar_entries", { projectId, startDate: dateKey(range.start), endDate: dateKey(range.end) })
      .then(setEntries)
      .catch((reason) => setMessage(reason instanceof Error ? reason.message : "Não foi possível carregar o Calendar."))
      .finally(() => setLoading(false));
  }, [projectId, range.end, range.start]);

  const addEntry = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!draft.trim()) return;
    if (!isTauriAvailable()) { setMessage("Abra o Orbit pelo aplicativo desktop para salvar o planejamento."); return; }
    try {
      const entry = await invoke<CalendarEntry>("create_calendar_entry", { input: { projectId, entryDate: selectedDate, content: draft } });
      setDraft("");
      if (entry.entryDate >= dateKey(range.start) && entry.entryDate <= dateKey(range.end)) setEntries((current) => [...current, entry]);
    } catch (reason) { setMessage(reason instanceof Error ? reason.message : "Não foi possível salvar o planejamento."); }
  };

  const monthStart = weekStart(new Date(selected.getFullYear(), selected.getMonth(), 1, 12));
  const monthEnd = addDays(weekStart(new Date(selected.getFullYear(), selected.getMonth() + 1, 0, 12)), 6);
  const monthDays = datesBetween(monthStart, monthEnd);
  const selectedEntries = entriesByDate[selectedDate] ?? [];

  return <AppShell activeSection="calendar" onNavigate={(section) => { window.location.hash = section === "home" ? "" : section === "canvas" ? `canvas/${encodeURIComponent(projectId)}` : section === "notes" ? `notes/${encodeURIComponent(projectId)}` : section === "calendar" ? `calendar/${encodeURIComponent(projectId)}` : section === "docs" ? `docs/${encodeURIComponent(projectId)}` : section; }}>
    <main className="calendar-page">
      <section className="calendar-content" aria-labelledby="calendar-title">
        <header className="calendar-titlebar">
          <div><p className="calendar-eyebrow">PLANEJAMENTO LOCAL</p><h1 id="calendar-title">Calendar</h1><p>Organize o que precisa ser feito por dia, semana ou mês.</p></div>
          <label className="calendar-date-picker">Data selecionada<input type="date" value={selectedDate} onChange={(event) => setSelectedDate(event.target.value)} /></label>
        </header>
        <div className="calendar-controls" aria-label="Modo de visualização"><div role="group" aria-label="Visualização do Calendar">{(["day", "week", "month"] as CalendarView[]).map((option) => <button key={option} type="button" className={view === option ? "active" : ""} onClick={() => setView(option)}>{option === "day" ? "Dia" : option === "week" ? "Semana" : "Mês"}</button>)}</div><strong>{view === "month" ? monthLabel.format(selected) : dayLabel.format(selected)}</strong></div>
        <section className="calendar-planner" aria-label="Planejamento">
          <form className="calendar-add-entry" onSubmit={(event) => void addEntry(event)}><label htmlFor="calendar-entry">Planejar para {dayLabel.format(selected)}<textarea id="calendar-entry" value={draft} onChange={(event) => setDraft(event.target.value)} placeholder="Ex.: revisar a arquitetura, estudar para a prova..." /></label><button type="submit" disabled={!draft.trim()}>Adicionar ao dia</button>{message && <p role="alert">{message}</p>}</form>
          <aside className="calendar-selected-day"><h2>{dayLabel.format(selected)}</h2><CalendarEntries entries={selectedEntries} selectedDate={selectedDate} onSelectDate={setSelectedDate} /></aside>
        </section>
        {loading ? <div className="calendar-state">Carregando planejamento...</div> : view === "day" ? <section className="calendar-day-view"><h2>{dayLabel.format(selected)}</h2><CalendarEntries entries={selectedEntries} selectedDate={selectedDate} onSelectDate={setSelectedDate} /></section> : view === "week" ? <section className="calendar-week-view" aria-label="Planejamento da semana">{periodDays.map((day) => { const key = dateKey(day); return <article key={key} className={key === selectedDate ? "selected" : ""}><button type="button" onClick={() => setSelectedDate(key)}><strong>{weekday.format(day)}</strong><span>{day.getDate()}</span></button><CalendarEntries entries={entriesByDate[key] ?? []} selectedDate={selectedDate} onSelectDate={setSelectedDate} /></article>; })}</section> : <section className="calendar-month-view" aria-label="Planejamento do mês"><div className="calendar-weekdays">{Array.from({ length: 7 }, (_, day) => <span key={day}>{weekday.format(addDays(weekStart(new Date(2024, 0, 1)), day)).replace(".", "")}</span>)}</div><div className="calendar-month-grid">{monthDays.map((day) => { const key = dateKey(day); const outsideMonth = day.getMonth() !== selected.getMonth(); return <article key={key} className={`${key === selectedDate ? "selected" : ""} ${outsideMonth ? "outside" : ""}`}><button type="button" onClick={() => setSelectedDate(key)} aria-label={`Selecionar ${dayLabel.format(day)}`}><span>{day.getDate()}</span></button>{(entriesByDate[key] ?? []).slice(0, 3).map((entry) => <button key={entry.id} type="button" className="calendar-entry-chip" onClick={() => setSelectedDate(key)}>{entry.content}</button>)}</article>; })}</div></section>}
      </section>
    </main>
  </AppShell>;
}
