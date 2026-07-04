function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

function highlightLine(line: string): string {
    const commentIndex = line.indexOf('#');
    const code = commentIndex === -1 ? line : line.slice(0, commentIndex);
    const comment = commentIndex === -1 ? '' : line.slice(commentIndex);

    const leadingSpaces = code.match(/^\s*/)?.[0] ?? '';
    const rest = code.slice(leadingSpaces.length);

    const dashMatch = rest.match(/^-\s*/);
    const dashPrefix = dashMatch ? dashMatch[0] : '';
    const afterDash = rest.slice(dashPrefix.length);

    const keyMatch = afterDash.match(/^([A-Za-z0-9_.-]+)(:)(\s|$)/);
    let highlighted: string;

    if (keyMatch) {
        const key = keyMatch[1];
        const colon = keyMatch[2];
        const separator = keyMatch[3];
        const value = afterDash.slice(keyMatch[0].length);
        highlighted = `<span class="text-sky-400">${escapeHtml(key)}</span><span class="text-muted-foreground">${colon}</span>${separator}${highlightValue(value)}`;
    } else {
        highlighted = highlightValue(afterDash);
    }

    const dashHtml = dashPrefix ? `<span class="text-muted-foreground">${escapeHtml(dashPrefix)}</span>` : '';
    const commentHtml = comment ? `<span class="text-emerald-600/70">${escapeHtml(comment)}</span>` : '';

    return `${escapeHtml(leadingSpaces)}${dashHtml}${highlighted}${commentHtml}`;
}

function highlightValue(value: string): string {
    if (!value) return '';
    if (/^["'].*["']$/.test(value)) {
        return `<span class="text-amber-400">${escapeHtml(value)}</span>`;
    }
    if (/^-?\d+(\.\d+)?$/.test(value)) {
        return `<span class="text-violet-400">${escapeHtml(value)}</span>`;
    }
    if (value === 'true' || value === 'false' || value === 'null') {
        return `<span class="text-violet-400">${escapeHtml(value)}</span>`;
    }
    return `<span class="text-foreground">${escapeHtml(value)}</span>`;
}

export function highlightYaml(yaml: string): string {
    return yaml.split('\n').map(highlightLine).join('\n');
}
