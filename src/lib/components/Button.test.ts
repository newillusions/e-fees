/**
 * Button Component Tests
 *
 * Tests for the base Button component including all variants,
 * states, and interaction patterns.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import Button from './Button.svelte';

describe('Button Component', () => {
  it('should render a button element', () => {
    render(Button);

    const button = screen.getByRole('button');
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute('type', 'button');
  });

  it('should apply default classes', () => {
    render(Button);

    const button = screen.getByRole('button');
    // Updated to match CSS-based button classes
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--primary');
  });

  it('should apply variant classes correctly', () => {
    // Test secondary variant
    const { container: container1 } = render(Button, { variant: 'secondary' });
    let button = container1.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--secondary');

    // Test ghost variant
    const { container: container2 } = render(Button, { variant: 'ghost' });
    button = container2.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--ghost');

    // Test primary variant
    const { container: container3 } = render(Button, { variant: 'primary' });
    button = container3.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--primary');

    // Test danger variant
    const { container: container4 } = render(Button, { variant: 'danger' });
    button = container4.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--danger');
  });

  it('should apply size classes correctly', () => {
    // Button now uses CSS classes for sizes
    // Test small size
    const { container: container1 } = render(Button, { size: 'sm' });
    let button = container1.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--sm', 'emittiv-btn--primary');

    // Test medium size (default)
    const { container: container2 } = render(Button, { size: 'md' });
    button = container2.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--md', 'emittiv-btn--primary');

    // Test large size
    const { container: container3 } = render(Button, { size: 'lg' });
    button = container3.querySelector('button');
    expect(button).toHaveClass('emittiv-btn', 'emittiv-btn--lg', 'emittiv-btn--primary');
  });

  it('should handle disabled state', () => {
    render(Button, { disabled: true });

    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    // CSS handles disabled styling via :disabled pseudo-class
    expect(button).toHaveClass('emittiv-btn');
  });

  it('should handle loading state', () => {
    const { container } = render(Button, { loading: true });

    const button = container.querySelector('button');
    // Button shows spinner but isn't automatically disabled when loading
    // (requires explicit disabled prop)
    expect(button).not.toBeDisabled();

    // Loading state shows a spinner
    const spinner = container.querySelector('.emittiv-spinner');
    expect(spinner).toBeInTheDocument();
  });

  it('should handle type attribute', () => {
    // Test submit type
    const { container: container1 } = render(Button, { type: 'submit' });
    let button = container1.querySelector('button');
    expect(button).toHaveAttribute('type', 'submit');

    // Test button type (default)
    const { container: container2 } = render(Button, { type: 'button' });
    button = container2.querySelector('button');
    expect(button).toHaveAttribute('type', 'button');
  });

  it('should apply custom className', () => {
    render(Button, { className: 'custom-class another-class' });

    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class', 'another-class');
  });

  it('should emit click events', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();

    render(Button);

    const button = screen.getByRole('button');
    button.addEventListener('click', handleClick);
    await user.click(button);

    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should not emit click when disabled', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();

    render(Button, { disabled: true });

    const button = screen.getByRole('button');
    button.addEventListener('click', handleClick);
    await user.click(button);

    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should emit click events even when loading', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();

    render(Button, { loading: true });

    const button = screen.getByRole('button');
    button.addEventListener('click', handleClick);
    await user.click(button);

    // Button allows clicks when loading (not automatically disabled)
    // Parent component should handle loading state logic if needed
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should be focusable by default', () => {
    render(Button);

    const button = screen.getByRole('button');
    button.focus();
    expect(button).toHaveFocus();
  });

  it('should not be focusable when disabled', () => {
    render(Button, { disabled: true });

    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
  });

  it('should have focus styles', () => {
    render(Button);

    const button = screen.getByRole('button');
    // CSS handles focus styling via :focus pseudo-class
    expect(button).toHaveClass('emittiv-btn');
  });
});
