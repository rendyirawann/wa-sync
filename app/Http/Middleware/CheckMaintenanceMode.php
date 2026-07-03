<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use App\Models\Setting;
use Illuminate\Support\Facades\Auth;

class CheckMaintenanceMode
{
    /**
     * Handle an incoming request.
     */
    public function handle(Request $request, Closure $next)
    {
        try {
            // Check if maintenance mode is enabled in settings
            if (Setting::get('maintenance_mode', '0') === '1') {
                
                // Allow superadmins to bypass
                if (Auth::check() && Auth::user()->hasRole(['Superadmin', 'superadmin'])) {
                    return $next($request);
                }

                // Allow access to auth routes so admins can still log in
                if ($request->is('login') || $request->is('logout') || $request->is('admin/login') || $request->is('admin/logout') || $request->is('admin/auth/*') || $request->routeIs('login') || $request->routeIs('logout')) {
                    return $next($request);
                }

                // Return 503 Maintenance response
                return response()->view('errors.503', [], 503);
            }
        } catch (\Exception $e) {
            // If DB/Settings fail, just continue normally
        }

        return $next($request);
    }
}
