import Adw from 'gi://Adw';
import Gtk from 'gi://Gtk';
import Gio from 'gi://Gio';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class GnomeShotPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();

        // Create a preferences page
        const page = new Adw.PreferencesPage({
            title: 'GNOME Shot',
            icon_name: 'camera-photo-symbolic',
        });
        window.add(page);

        // Keyboard shortcuts group
        const shortcutsGroup = new Adw.PreferencesGroup({
            title: 'Keyboard Shortcuts',
            description: 'Configure keyboard shortcuts for GNOME Shot',
        });
        page.add(shortcutsGroup);

        // Capture shortcut row
        const captureRow = new Adw.ActionRow({
            title: 'Capture Screenshot',
            subtitle: 'Opens GNOME Shot to capture a screenshot',
        });

        const shortcutLabel = new Gtk.ShortcutLabel({
            accelerator: settings.get_strv('capture-shortcut')[0] || '',
            valign: Gtk.Align.CENTER,
        });

        const editButton = new Gtk.Button({
            icon_name: 'document-edit-symbolic',
            valign: Gtk.Align.CENTER,
            css_classes: ['flat'],
        });

        editButton.connect('clicked', () => {
            this._showShortcutDialog(window, settings, shortcutLabel);
        });

        captureRow.add_suffix(shortcutLabel);
        captureRow.add_suffix(editButton);
        shortcutsGroup.add(captureRow);

        // Info group
        const infoGroup = new Adw.PreferencesGroup({
            title: 'About',
        });
        page.add(infoGroup);

        const infoRow = new Adw.ActionRow({
            title: 'GNOME Shot',
            subtitle: 'Screenshot and annotation tool for GNOME',
        });
        infoGroup.add(infoRow);
    }

    _showShortcutDialog(window, settings, shortcutLabel) {
        const dialog = new Gtk.Dialog({
            title: 'Set Shortcut',
            transient_for: window,
            modal: true,
            default_width: 400,
            default_height: 200,
        });

        const contentArea = dialog.get_content_area();
        contentArea.set_margin_top(20);
        contentArea.set_margin_bottom(20);
        contentArea.set_margin_start(20);
        contentArea.set_margin_end(20);

        const label = new Gtk.Label({
            label: 'Press a key combination...\n\nPress Escape to cancel, Backspace to clear',
            halign: Gtk.Align.CENTER,
        });
        contentArea.append(label);

        const controller = new Gtk.EventControllerKey();
        controller.connect('key-pressed', (controller, keyval, keycode, state) => {
            // Remove lock modifiers
            state = state & Gtk.accelerator_get_default_mod_mask();

            if (keyval === Gdk.KEY_Escape) {
                dialog.close();
                return true;
            }

            if (keyval === Gdk.KEY_BackSpace) {
                settings.set_strv('capture-shortcut', []);
                shortcutLabel.set_accelerator('');
                dialog.close();
                return true;
            }

            if (Gtk.accelerator_valid(keyval, state)) {
                const accel = Gtk.accelerator_name(keyval, state);
                settings.set_strv('capture-shortcut', [accel]);
                shortcutLabel.set_accelerator(accel);
                dialog.close();
            }

            return true;
        });

        dialog.add_controller(controller);
        dialog.present();
    }
}
