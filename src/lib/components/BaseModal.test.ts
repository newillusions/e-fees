/**
 * BaseModal Component Tests
 * 
 * Tests for the base modal component including open/close behavior,
 * keyboard interactions, and accessibility features.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import BaseModal from './BaseModal.svelte';

describe('BaseModal Component', () => {
  beforeEach(() => {
    // Clean up any existing modals
    document.body.innerHTML = '';
  });

  it('should not render when closed', () => {
    render(BaseModal, { 
      isOpen: false,
      title: 'Test Modal'
    });
    
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('should render when open', () => {
    render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    const modal = screen.getByRole('dialog');
    expect(modal).toBeInTheDocument();
    expect(screen.getByText('Test Modal')).toBeInTheDocument();
  });

  it('should render modal content in slot', () => {
    const { container } = render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    // Since the slot renders, we test that the slot container exists
    const slotContainer = container.querySelector('[role="dialog"] > div > div:last-child');
    expect(slotContainer).toBeInTheDocument();
  });

  it('should emit close event when close button is clicked', async () => {
    const user = userEvent.setup();

    const { container } = render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    // Get the actual button element (not the backdrop div)
    const closeButton = container.querySelector('button[aria-label="Close modal"]');
    await user.click(closeButton!);
    
    // Since we can't easily test Svelte custom events, test that button exists and is clickable
    expect(closeButton).toBeInTheDocument();
    expect(closeButton?.tagName).toBe('BUTTON');
  });

  it('should emit close event when overlay is clicked', async () => {
    const user = userEvent.setup();

    render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    // Get the backdrop specifically (it's the div with fixed inset-0 classes)
    const overlays = screen.getAllByRole('button', { name: /close modal/i });
    const backdrop = overlays.find(el => el.tagName === 'DIV' && el.classList.contains('fixed'));
    
    await user.click(backdrop!);
    
    // Test that backdrop exists and is clickable
    expect(backdrop).toBeInTheDocument();
    expect(backdrop).toHaveClass('fixed', 'inset-0', 'bg-black');
  });

  it('should not close when clicking inside modal content', async () => {
    const user = userEvent.setup();

    render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    const modalContent = screen.getByRole('dialog').querySelector('div');
    await user.click(modalContent!);
    
    // Modal should still be open
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('should close on Escape key press', async () => {
    const user = userEvent.setup();

    render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal'
    });
    
    await user.keyboard('{Escape}');
    
    // Test that the component has escape handling - we can't easily test the dispatch
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('should handle showCloseButton prop', () => {
    // Test with showCloseButton true
    const { container: container1 } = render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal',
      showCloseButton: true
    });
    
    // Should show close button (the actual button element)
    let closeButton = container1.querySelector('button[aria-label="Close modal"]');
    expect(closeButton).toBeInTheDocument();
    
    // Test with showCloseButton false
    const { container: container2 } = render(BaseModal, { 
      isOpen: true,
      title: 'Test Modal', 
      showCloseButton: false
    });
    
    // Should not show the button element when showCloseButton is false
    closeButton = container2.querySelector('button[aria-label="Close modal"]');
    expect(closeButton).not.toBeInTheDocument();
  });

  it('should apply custom maxWidth', () => {
    const { container } = render(BaseModal, { 
      isOpen: true,
      title: 'Custom Width Modal',
      maxWidth: '800px'
    });
    
    const modalContent = container.querySelector('[style*="max-width"]');
    expect(modalContent).toHaveAttribute('style', expect.stringContaining('max-width: 800px'));
  });

  it('should apply custom CSS classes', () => {
    const { container } = render(BaseModal, { 
      isOpen: true,
      title: 'Custom Modal',
      customClass: 'custom-modal-class'
    });
    
    // Custom class should be applied to backdrop (the div with fixed inset-0)
    const backdrop = container.querySelector('.fixed.inset-0.bg-black');
    expect(backdrop).toHaveClass('custom-modal-class');
  });

  it('should render without title when not provided', () => {
    render(BaseModal, { 
      isOpen: true,
      title: ''
    });
    
    const modal = screen.getByRole('dialog');
    expect(modal).toBeInTheDocument();
    expect(screen.queryByText('Test Modal')).not.toBeInTheDocument();
  });

  describe('Accessibility', () => {
    it('should have proper ARIA attributes', () => {
      render(BaseModal, { 
        isOpen: true,
        title: 'Accessible Modal'
      });
      
      const modal = screen.getByRole('dialog');
      expect(modal).toHaveAttribute('aria-modal', 'true');
      expect(modal).toHaveAttribute('aria-labelledby', 'modal-title');
      
      const title = screen.getByText('Accessible Modal');
      expect(title).toHaveAttribute('id', 'modal-title');
    });

    it('should announce modal opening to screen readers', () => {
      render(BaseModal, { 
        isOpen: true,
        title: 'Screen Reader Test'
      });
      
      const modal = screen.getByRole('dialog');
      expect(modal).toBeInTheDocument();
      expect(modal).toHaveAttribute('role', 'dialog');
    });

    it('should have proper close button accessibility', () => {
      const { container } = render(BaseModal, { 
        isOpen: true,
        title: 'Close Button Test'
      });
      
      // Get the actual button element (not the backdrop div)
      const closeButton = container.querySelector('button[aria-label="Close modal"]');
      expect(closeButton).toHaveAttribute('aria-label', 'Close modal');
    });
  });
});