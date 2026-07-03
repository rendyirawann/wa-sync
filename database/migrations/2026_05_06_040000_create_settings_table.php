<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;
use Illuminate\Support\Facades\DB;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('settings', function (Blueprint $table) {
            $table->id();
            $table->string('key')->unique();
            $table->text('value')->nullable();
            $table->timestamps();
        });

        // Seed default settings
        $defaults = [
            ['key' => 'site_logo', 'value' => 'base-logo.png'],
            ['key' => 'site_name', 'value' => 'StarterTemp'],
            ['key' => 'site_font', 'value' => 'Plus Jakarta Sans'],

            ['key' => 'social_google_enabled', 'value' => '0'],
            ['key' => 'social_google_client_id', 'value' => ''],
            ['key' => 'social_google_client_secret', 'value' => ''],

            ['key' => 'social_facebook_enabled', 'value' => '0'],
            ['key' => 'social_facebook_client_id', 'value' => ''],
            ['key' => 'social_facebook_client_secret', 'value' => ''],

            ['key' => 'social_github_enabled', 'value' => '0'],
            ['key' => 'social_github_client_id', 'value' => ''],
            ['key' => 'social_github_client_secret', 'value' => ''],

            ['key' => 'social_linkedin_enabled', 'value' => '0'],
            ['key' => 'social_linkedin_client_id', 'value' => ''],
            ['key' => 'social_linkedin_client_secret', 'value' => ''],
        ];

        foreach ($defaults as $setting) {
            DB::table('settings')->insert(array_merge($setting, [
                'created_at' => now(),
                'updated_at' => now(),
            ]));
        }
    }

    public function down(): void
    {
        Schema::dropIfExists('settings');
    }
};
