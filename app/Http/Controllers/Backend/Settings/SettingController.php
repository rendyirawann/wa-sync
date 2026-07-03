<?php

namespace App\Http\Controllers\Backend\Settings;

use App\Http\Controllers\Controller;
use App\Models\Setting;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Storage;

class SettingController extends Controller
{
    public function index()
    {
        $settings = Setting::allCached();
        $fonts = [
            'Plus Jakarta Sans' => 'Plus+Jakarta+Sans:wght@300;400;500;600;700;800',
            'Inter' => 'Inter:wght@300;400;500;600;700',
            'Outfit' => 'Outfit:wght@300;400;500;600;700;800',
            'Poppins' => 'Poppins:wght@300;400;500;600;700',
            'DM Sans' => 'DM+Sans:wght@300;400;500;600;700',
            'Nunito' => 'Nunito:wght@300;400;500;600;700;800',
            'Figtree' => 'Figtree:wght@300;400;500;600;700;800',
            'Manrope' => 'Manrope:wght@300;400;500;600;700;800',
        ];
        return view('backend.settings.index', compact('settings', 'fonts'));
    }

    public function update(Request $request)
    {
        // --- General Settings ---
        if ($request->has('site_name')) {
            Setting::set('site_name', $request->input('site_name'));
        }

        if ($request->has('site_font')) {
            Setting::set('site_font', $request->input('site_font'));
        }

        Setting::set('maintenance_mode', $request->has('maintenance_mode') ? '1' : '0');

        // --- Logo Upload ---
        if ($request->hasFile('site_logo')) {
            $request->validate(['site_logo' => 'image|mimes:png,jpg,jpeg,svg,webp|max:2048']);

            $file = $request->file('site_logo');
            $filename = 'site-logo-' . time() . '.' . $file->getClientOriginalExtension();

            // Store in public/assets/media/logos/
            $file->move(public_path('assets/media/logos'), $filename);

            // Delete old logo if it's not the default
            $oldLogo = Setting::get('site_logo', 'base-logo.png');
            if ($oldLogo !== 'base-logo.png' && file_exists(public_path('assets/media/logos/' . $oldLogo))) {
                unlink(public_path('assets/media/logos/' . $oldLogo));
            }

            Setting::set('site_logo', $filename);
        }

        // --- Social Login Settings ---
        $providers = ['google', 'facebook', 'github', 'linkedin'];
        foreach ($providers as $provider) {
            Setting::set("social_{$provider}_enabled", $request->has("social_{$provider}_enabled") ? '1' : '0');

            if ($request->has("social_{$provider}_client_id")) {
                Setting::set("social_{$provider}_client_id", $request->input("social_{$provider}_client_id"));
            }
            if ($request->has("social_{$provider}_client_secret")) {
                $secret = $request->input("social_{$provider}_client_secret");
                // Only update secret if not empty (user might leave it blank to keep existing)
                if (!empty($secret)) {
                    Setting::set("social_{$provider}_client_secret", $secret);
                }
            }
        }

        Setting::clearCache();

        return redirect()->route('settings.index')->with('success', 'Settings berhasil diperbarui!');
    }
}
