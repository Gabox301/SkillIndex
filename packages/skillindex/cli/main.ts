import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { cleanupClaudeMd } from './claude.ts';
import { bold, cyan, dim, gray, green, log, magenta, muted, red, SHOW_CURSOR, write, yellow } from './colors.ts';
import type { InstallSecurityCheck } from './installer.ts';
import {
  agentFolderFor,
  clearSkillIndexCache,
  installAll,
  loadRegistry,
  securityCheckForSkillPath,
} from './installer.ts';
import type { ComboSkill, SkillEntry, Technology } from './lib.ts';
import { collectSkills, detectAgents, detectTechnologies, getInstalledSkillNames, partitionCombos } from './lib.ts';
import { formatTime, multiSelect, printBanner } from './ui.ts';

const __dirname = import.meta.dirname;
const VERSION: string = (() => {
  for (const base of [__dirname, resolve(__dirname, '..'), resolve(__dirname, '..', '..')]) {
    const p = join(base, 'package.json');
    if (!existsSync(p)) continue;
    try {
      const pkg = JSON.parse(readFileSync(p, 'utf-8'));
      if (typeof pkg.name === 'string' && pkg.name.toLowerCase() === 'skillindex') return pkg.version;
    } catch {}
  }
  return '0.0.0';
})();
const ISSUES_URL = 'https://github.com/Gabox301/SkillIndex/issues';

process.on('SIGINT', () => {
  write(SHOW_CURSOR + '\n');
  process.exit(130);
});

// ── CLI ──────────────────────────────────────────────────────

interface CliArgs {
  autoYes: boolean;
  dryRun: boolean;
  verbose: boolean;
  help: boolean;
  clearCache: boolean;
  agents: string[];
  security: boolean;
}

function parseArgs(): CliArgs {
  const args = process.argv.slice(2);
  const agents: string[] = [];
  const agentIdx = args.findIndex((a) => a === '-a' || a === '--agent');
  if (agentIdx !== -1) {
    for (let i = agentIdx + 1; i < args.length; i++) {
      if (args[i].startsWith('-')) break;
      agents.push(args[i]);
    }
  }
  return {
    autoYes: args.includes('-y') || args.includes('--yes'),
    dryRun: args.includes('--dry-run'),
    verbose: args.includes('--verbose') || args.includes('-v'),
    help: args.includes('--help') || args.includes('-h'),
    clearCache: args.includes('--clear-cache'),
    agents,
    security: args.includes('--security'),
  };
}

function showHelp(): void {
  log(`
  ${bold('skillindex')} — Instala las mejores skills de IA para tu proyecto
  ${bold('Uso:')}
    npx skillindex                   Detecta e instala skills
    npx skillindex ${dim('-y')}                   Omitir confirmación
    npx skillindex ${dim('--dry-run')}            Mostrar qué se instalaría sin instalar
    npx skillindex ${dim('--clear-cache')}        Limpiar caché de skills descargadas
    npx skillindex ${dim('-a cursor claude-code')} Instalar solo para IDEs específicos
    npx skillindex ${dim('--security')}            Incluir combos de seguridad opcionales
  ${bold('Opciones:')}
    -y, --yes       Omitir confirmación
    --dry-run       Mostrar qué se instalaría sin instalar
    --clear-cache   Limpiar caché de skills descargadas
    --security      Incluir combos de seguridad opcionales (sin preguntar)
    -v, --verbose   Mostrar traza de instalación y detalles de error
    -a, --agent     Instalar solo para IDEs específicos (ej. cursor, claude-code)
    -h, --help      Mostrar esta ayuda
 `);
}

// ── Display ──────────────────────────────────────────────────

function printDetected(detected: Technology[], combos: ComboSkill[], isFrontend: boolean): void {
  if (detected.length > 0) {
    const withSkills = detected.filter((t) => t.skills.length > 0);
    const withoutSkills = detected.filter((t) => t.skills.length === 0);
    const allTech = [...withSkills, ...withoutSkills];
    log(cyan('   ◆ ') + bold('Tecnologías detectadas:'));
    log();
    const COLS = 3;
    const colWidth = Math.max(...allTech.map((t) => t.name.length)) + 3;
    const formatTech = (tech: Technology): string => {
      const hasSkills = tech.skills.length > 0;
      const icon = hasSkills ? green('✔') : dim('●');
      const name = tech.name.padEnd(colWidth);
      return `${icon} ${hasSkills ? name : dim(name)}`;
    };
    for (let i = 0; i < allTech.length; i += COLS) {
      const row = allTech
        .slice(i, i + COLS)
        .map(formatTech)
        .join('');
      log(`     ${row}`);
    }
    if (combos.length > 0) {
      log();
      log(magenta('   ◆ ') + bold('Combinaciones detectadas:'));
      log();
      for (const combo of combos) {
        log(magenta(`     ⚡ `) + combo.name);
      }
    }
    log();
  }
  if (isFrontend && detected.length === 0) {
    log(cyan('   ◆ ') + bold('Frontend web detectado ') + dim('(a partir de archivos del proyecto)'));
    log();
  }
}

function formatSkillLabel(skill: string, { styled = false }: { styled?: boolean } = {}): string {
  if (/^https?:\/\//i.test(skill)) {
    return styled ? cyan(skill) : skill;
  }
  const parts = skill.split('/');
  if (parts.length !== 3) {
    return styled ? cyan(skill) : skill;
  }
  const [author, , skillName] = parts;
  if (!styled) {
    return `${author} › ${skillName}`;
  }
  return `${muted(author)} ${gray('›')} ${cyan(bold(skillName))}`;
}

function securityWarningForSkill(skill: string): string | null {
  const check = securityCheckForSkillPath(skill);
  if (check?.status !== 'warning') return null;
  const findings = check.findings.map((finding) => finding.trim()).filter(Boolean);
  const detail = [check.summary.trim(), findings.join('; ')].filter(Boolean).join(' ');
  return detail || 'La revisión de sincronización encontró observaciones que deberías revisar.';
}

function printSkillsList(skills: SkillEntry[]): void {
  const INSTALLED_TAG = ' (instalada)';
  const SECURITY_TAG = ' (revisión de seguridad ⚠)';
  const entries = skills.map((s) => ({
    ...s,
    label: formatSkillLabel(s.skill),
    styledLabel: formatSkillLabel(s.skill, { styled: true }),
    hasSecurityWarning: Boolean(securityWarningForSkill(s.skill)),
  }));
  const maxEffective = Math.max(
    ...entries.map(
      (e) =>
        e.label.length + (e.installed ? INSTALLED_TAG.length : 0) + (e.hasSecurityWarning ? SECURITY_TAG.length : 0),
    ),
  );
  const newCount = skills.filter((s) => !s.installed).length;
  const installedCount = skills.length - newCount;
  const countLabel = installedCount > 0 ? `(${skills.length}, ${installedCount} ya instaladas)` : `(${skills.length})`;
  log(cyan('   ◆ ') + bold(`Skills por instalar `) + dim(countLabel));
  log();
  for (let i = 0; i < entries.length; i++) {
    const { label, styledLabel, sources, installed, hasSecurityWarning } = entries[i];
    const techSources = sources.filter((s) => !s.includes(' + '));
    const installedTag = installed ? dim(INSTALLED_TAG) : '';
    const securityTag = hasSecurityWarning ? yellow(SECURITY_TAG) : '';
    const effectiveLen =
      label.length + (installed ? INSTALLED_TAG.length : 0) + (hasSecurityWarning ? SECURITY_TAG.length : 0);
    const pad = ' '.repeat(maxEffective - effectiveLen);
    const num = String(i + 1).padStart(2, ' ');
    const sourceSuffix = techSources.length > 0 ? `  ${dim(`← ${techSources.join(', ')}`)}` : '';
    log(dim(`   ${num}.`) + ` ${styledLabel}${installedTag}${securityTag}${pad}${sourceSuffix}`);
  }
  log();
}

function stripAnsi(str: string): string {
  const esc = String.fromCharCode(27);
  return str.replace(new RegExp(esc + '\\[[0-9;]*m', 'g'), '');
}

function extractErrorLines(stderr: string, output: string): string[] {
  const raw = stderr?.trim() || output?.trim() || '';
  const noisePatterns = [
    /^npm\s+(warn|notice|http)\b/i,
    /^npm\s+error\s*$/i,
    /^\s*$/,
    /^>\s/,
    /^added\s+\d+\s+packages/i,
    /^up to date/i,
    /^npm error A complete log of this run/i,
    /^npm error\s+[\w/\\:.-]+debug-\d+\.log$/i,
  ];
  return stripAnsi(raw)
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l && !noisePatterns.some((p) => p.test(l)));
}

function briefErrorReason(stderr: string, output: string): string {
  const lines = extractErrorLines(stderr, output);
  if (lines.length === 0) return 'Error desconocido';
  const line = lines[0];
  return line.length > 80 ? line.slice(0, 77) + '...' : line;
}

function visiblePad(value: string, width: number): string {
  return value + ' '.repeat(Math.max(0, width - stripAnsi(value).length));
}

function truncateVisible(value: string, width: number): string {
  const plain = stripAnsi(value);
  if (plain.length <= width) return value;
  if (width <= 1) return '…';
  return plain.slice(0, width - 1) + '…';
}

function wrapText(value: string, width: number): string[] {
  if (width <= 0) return [value];
  const words = value.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return [''];
  const lines: string[] = [];
  let line = '';
  for (const word of words) {
    if (word.length > width) {
      if (line) {
        lines.push(line);
        line = '';
      }
      for (let i = 0; i < word.length; i += width) {
        lines.push(word.slice(i, i + width));
      }
      continue;
    }
    const next = line ? `${line} ${word}` : word;
    if (next.length > width) {
      lines.push(line);
      line = word;
    } else {
      line = next;
    }
  }
  if (line) lines.push(line);
  return lines;
}

function formatSecurityFindings(check: InstallSecurityCheck): string | null {
  const findings = check.findings.map((finding) => finding.trim()).filter(Boolean);
  if (findings.length === 0) return null;
  const summary = check.summary.trim();
  return [summary, findings.join('; ')].filter(Boolean).join(' ');
}

function printSecurityChecks(checks: InstallSecurityCheck[]): void {
  const checksWithFindings = checks
    .map((check) => ({ check, findings: formatSecurityFindings(check) }))
    .filter((entry): entry is { check: InstallSecurityCheck; findings: string } => Boolean(entry.findings));
  if (checksWithFindings.length === 0) return;
  const sorted = checksWithFindings.sort((a, b) => a.check.name.localeCompare(b.check.name));
  const skillWidth = Math.min(34, Math.max(5, ...sorted.map(({ check }) => check.name.length)));
  const checkWidth = 12;
  const terminalWidth = process.stdout.columns || 100;
  const findingsWidth = Math.max(40, terminalWidth - skillWidth - checkWidth - 16);
  log();
  log(cyan('   ◆ ') + bold('Verificaciones de seguridad'));
  log();
  log(
    dim(
      `   | ${visiblePad('Skill', skillWidth)} | ${visiblePad('Verificación', checkWidth)} | ${visiblePad('Hallazgos', findingsWidth)} |`,
    ),
  );
  log(dim(`   | ${'-'.repeat(skillWidth)} | ${'-'.repeat(checkWidth)} | ${'-'.repeat(findingsWidth)} |`));
  for (const { check, findings } of sorted) {
    const status = check.status === 'warning' ? yellow('advertencia') : green('ok');
    const lines = wrapText(findings, findingsWidth);
    log(
      `   | ${visiblePad(truncateVisible(check.name, skillWidth), skillWidth)} | ${visiblePad(status, checkWidth)} | ${visiblePad(lines[0], findingsWidth)} |`,
    );
    for (const line of lines.slice(1)) {
      log(`   | ${visiblePad('', skillWidth)} | ${visiblePad('', checkWidth)} | ${visiblePad(line, findingsWidth)} |`);
    }
  }
}

interface SummaryOptions {
  installed: number;
  failed: number;
  errors: {
    name: string;
    output: string;
    stderr: string;
    exitCode: number | null;
    command: string;
  }[];
  elapsed: number;
  verbose: boolean;
}

function printSummary({ installed, failed, errors, elapsed, verbose }: SummaryOptions): void {
  log();
  if (failed === 0) {
    log(
      green(
        bold(
          `   ✔ ¡Listo! ${installed} skill${installed !== 1 ? 's' : ''} instalad${installed !== 1 ? 'as' : 'a'} en ${formatTime(elapsed)}.`,
        ),
      ),
    );
  } else {
    log(
      yellow(
        `   Completado: ${green(`${installed} instaladas`)}, ${red(`${failed} con error`)} en ${formatTime(elapsed)}.`,
      ),
    );
    if (errors.length > 0) {
      log();
      log(bold(red('   Errores:')));
      for (const { name, output, stderr, exitCode, command } of errors) {
        log(red(`     ✘ ${name}`));
        if (verbose) {
          if (exitCode !== undefined && exitCode !== null) {
            log(dim(`       código de salida ${exitCode}`));
          }
          const errorLines = extractErrorLines(stderr, output);
          if (errorLines.length > 0) {
            log();
            for (const line of errorLines.slice(0, 20)) {
              log(dim(`       ${line}`));
            }
            if (errorLines.length > 20) {
              log(dim(`       … (${errorLines.length - 20} líneas más)`));
            }
          }
          if (command) {
            log();
            log(dim(`       comando: ${command}`));
          }
          log();
        } else {
          const reason = briefErrorReason(stderr, output);
          log(dim(`       ${reason}`));
        }
      }
      log();
      if (!verbose) {
        log(dim('   Ejecuta de nuevo con --verbose para ver los detalles completos del error.'));
      }
      log(dim(`   Si parece un error de skillindex, por favor crea un issue: ${ISSUES_URL}`));
    }
  }
  log();
}

// ── Security Optional ────────────────────────────────────────

async function askIncludeSecurity(
  securityCombos: ComboSkill[],
  autoYes: boolean,
  forceSecurity: boolean,
): Promise<boolean> {
  if (securityCombos.length === 0) return false;
  if (forceSecurity) return true;
  if (autoYes || !process.stdin.isTTY) return false;

  log(cyan('   ◆ ') + bold('Seguridad (opcionales)') + dim(` — ${securityCombos.length} combos`));
  log(dim(`   ${securityCombos.map((c) => c.name).join(' · ')}`));
  log(dim('   ¿Incluir skills de seguridad? Por defecto no. [y/N]'));
  log('');

  const rl = createInterface({ input: process.stdin, output: process.stdout });
  const answer: string = await new Promise((resolve) => {
    rl.question(dim('   ¿Incluir? [y/N]: '), (ans) => {
      rl.close();
      resolve(ans.trim().toLowerCase());
    });
  });
  const include = answer === 'y' || answer === 'yes' || answer === 's' || answer === 'si';
  log('');
  return include;
}

// ── Agent Selection ──────────────────────────────────────────

async function selectAgents(agents: string[], autoYes: boolean): Promise<string[]> {
  const realAgents = agents.filter((a) => a !== 'universal');
  if (realAgents.length <= 1) return agents;
  if (autoYes || !process.stdout.isTTY) return agents;

  log(cyan('   ◆ ') + bold('Selecciona dónde instalar') + dim(` (${realAgents.length} agentes detectados)`));
  log(dim('   Desmarca los que no quieras. Por defecto instala en todos.'));
  log('');

  const selected = await multiSelect(realAgents, {
    labelFn: (agent: string) => {
      const folder = agentFolderFor(agent) ?? '.agents';
      return `${bold(agent)} ${dim(`(${folder})`)}`;
    },
    initialSelected: realAgents.map(() => true),
  });

  if (selected.length === 0) {
    log('');
    log(dim('   Ningún agente seleccionado — no se instalará nada.'));
    log('');
    process.exit(0);
  }

  return selected;
}

// ── Skill Selection ──────────────────────────────────────────

async function selectSkills(skills: SkillEntry[], autoYes: boolean): Promise<SkillEntry[]> {
  if (autoYes) {
    printSkillsList(skills);
    return skills;
  }
  const INSTALLED_TAG = ' (instalada)';
  const SECURITY_TAG = ' (revisión de seguridad ⚠)';
  const labelCache = new Map<string, { label: string; styledLabel: string; hasSecurityWarning: boolean }>();
  for (const s of skills) {
    labelCache.set(s.skill, {
      label: formatSkillLabel(s.skill),
      styledLabel: formatSkillLabel(s.skill, { styled: true }),
      hasSecurityWarning: Boolean(securityWarningForSkill(s.skill)),
    });
  }
  const maxEffective = Math.max(
    ...skills.map((s) => {
      const cached = labelCache.get(s.skill)!;
      return (
        cached.label.length +
        (s.installed ? INSTALLED_TAG.length : 0) +
        (cached.hasSecurityWarning ? SECURITY_TAG.length : 0)
      );
    }),
  );
  const newCount = skills.filter((s) => !s.installed).length;
  const installedCount = skills.length - newCount;
  const countLabel =
    installedCount > 0
      ? `${skills.length} encontradas, ${installedCount} ya instaladas`
      : `${skills.length} encontradas`;
  log(cyan('   ◆ ') + bold(`Selecciona las skills a instalar `) + dim(`(${countLabel})`));
  log();
  const selected = await multiSelect(skills, {
    labelFn: (s) => {
      const { label, styledLabel, hasSecurityWarning } = labelCache.get(s.skill)!;
      const installedTag = s.installed ? ' ' + dim('(instalada)') : '';
      const securityTag = hasSecurityWarning ? yellow(SECURITY_TAG) : '';
      const effectiveLen =
        label.length + (s.installed ? INSTALLED_TAG.length : 0) + (hasSecurityWarning ? SECURITY_TAG.length : 0);
      return styledLabel + installedTag + securityTag + ' '.repeat(maxEffective - effectiveLen);
    },
    hintFn: (s) => {
      const techSources = s.sources.filter((src) => !src.includes(' + '));
      return techSources.length > 1 ? `← ${techSources.join(', ')}` : '';
    },
    groupFn: (s) => s.sources[0],
    initialSelected: skills.map((s) => !s.installed),
    shortcuts:
      installedCount > 0
        ? [
            { key: 'n', label: 'nuevas', fn: (items: SkillEntry[]) => items.map((s) => !s.installed) },
            {
              key: 'i',
              label: 'instaladas',
              fn: (items: SkillEntry[]) => items.map((s) => s.installed),
            },
          ]
        : [],
  });
  if (selected.length === 0) {
    log();
    log(dim('   Nada seleccionado.'));
    log();
    process.exit(0);
  }
  return selected;
}

// ── Main ─────────────────────────────────────────────────────

async function main(): Promise<void> {
  const { autoYes, dryRun, verbose, help, clearCache, agents, security } = parseArgs();
  if (help) {
    showHelp();
    process.exit(0);
  }
  if (clearCache) {
    const { cacheDir, removed } = clearSkillIndexCache();
    log(
      removed
        ? green(`   ✔ Caché de skillindex limpiada: ${cacheDir}`)
        : dim(`   No se encontró caché de skillindex: ${cacheDir}`),
    );
    log();
    process.exit(0);
  }
  await printBanner(VERSION);
  const projectDir = resolve('.');
  write(dim('   Analizando proyecto...\r'));
  const { detected, isFrontend, combos: allCombos } = detectTechnologies(projectDir);
  write('\x1b[K');
  const { regular: regularCombos, security: securityCombos } = partitionCombos(allCombos);

  // 1. Tecnologías detectadas (sin seguridad)
  printDetected(detected, regularCombos, isFrontend);

  // 2. Agentes — decidir dónde instalar
  let resolvedAgents = agents.length > 0 ? agents : detectAgents(projectDir);
  if (agents.length === 0) {
    resolvedAgents = await selectAgents(resolvedAgents, autoYes);
  }

  // 3. Seguridad (opcionales) — checkbox y/n
  const includeSecurity = await askIncludeSecurity(securityCombos, autoYes, security);
  const combos = includeSecurity ? [...regularCombos, ...securityCombos] : regularCombos;
  if (includeSecurity && securityCombos.length > 0) {
    log(dim(`   ↳ Seguridad incluida: ${securityCombos.map((c) => c.name).join(', ')}`));
    log('');
  }

  if (detected.length === 0 && !isFrontend && combos.length === 0) {
    log(yellow('   ⚠ No se detectaron tecnologías compatibles.'));
    log(dim('   Asegúrate de ejecutar esto en el directorio de un proyecto.'));
    log(dim('   Tip: activa Seguridad (opcionales) con --security si quieres skills de seguridad.'));
    log();
    process.exit(0);
  }

  // 4. Skills — con o sin seguridad según el check
  const installedNames = getInstalledSkillNames(projectDir);
  const skills = collectSkills({ detected, isFrontend, combos, installedNames });
  if (skills.length === 0) {
    log(yellow('   Aún no hay skills disponibles para tu stack.'));
    log(dim('   Consulta https://skillindex.netlify.app para las últimas novedades.'));
    log();
    process.exit(0);
  }
  if (!dryRun) {
    setImmediate(loadRegistry);
  }
  if (dryRun) {
    printSkillsList(skills);
    log(dim(`   Agentes: ${resolvedAgents.join(', ')}`));
    log(dim('   --dry-run: no se instaló nada.'));
    log();
    process.exit(0);
  }
  const selectedSkills = await selectSkills(skills, autoYes);
  log();
  log(cyan('   ◆ ') + bold('Instalando skills...'));
  log(dim(`   Agentes: ${resolvedAgents.join(', ')}`));
  log();
  const startTime = Date.now();
  const { installed, failed, errors, securityChecks } = await installAll(selectedSkills, resolvedAgents, {
    verbose,
  });
  const elapsed = Date.now() - startTime;
  const claudeCleanup = cleanupClaudeMd(projectDir);
  if (claudeCleanup.cleaned) {
    if (claudeCleanup.deleted) {
      log(dim('   Se eliminó la sección de skillindex de CLAUDE.md (archivo vacío, eliminado).'));
    } else {
      log(dim('   Se eliminó la sección de skillindex de CLAUDE.md.'));
    }
    log();
  }
  printSecurityChecks(securityChecks);
  printSummary({ installed, failed, errors, elapsed, verbose });
}

main().catch((err: Error) => {
  console.error(red(`\n   Error: ${err.message}\n`));
  process.exit(1);
});
