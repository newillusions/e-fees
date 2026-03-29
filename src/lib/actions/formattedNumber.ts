import { formatNumber, parseCurrency } from '$lib/utils/format';

interface FormattedNumberOptions {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
}

/**
 * Svelte action for number inputs with comma-separated formatting.
 * Shows formatted value (e.g., "26,316") when not focused.
 * Shows raw number when focused for easy editing.
 * Parses and reformats on blur.
 */
export function formattedNumber(node: HTMLInputElement, options: FormattedNumberOptions) {
  let currentOptions = options;

  function format() {
    if (document.activeElement !== node) {
      node.value = formatNumber(currentOptions.value);
    }
  }

  function handleFocus() {
    // Show raw number for editing
    node.value = currentOptions.value === 0 ? '' : String(currentOptions.value);
    node.select();
  }

  function handleBlur() {
    const parsed = parseCurrency(node.value);
    const clamped = clamp(parsed);
    currentOptions.onChange(clamped);
    node.value = formatNumber(clamped);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      node.blur();
    }
  }

  function clamp(value: number): number {
    let v = value;
    if (currentOptions.min !== undefined && v < currentOptions.min) v = currentOptions.min;
    if (currentOptions.max !== undefined && v > currentOptions.max) v = currentOptions.max;
    return v;
  }

  // Set initial formatted value
  format();

  node.addEventListener('focus', handleFocus);
  node.addEventListener('blur', handleBlur);
  node.addEventListener('keydown', handleKeydown);

  return {
    update(newOptions: FormattedNumberOptions) {
      currentOptions = newOptions;
      format();
    },
    destroy() {
      node.removeEventListener('focus', handleFocus);
      node.removeEventListener('blur', handleBlur);
      node.removeEventListener('keydown', handleKeydown);
    }
  };
}
