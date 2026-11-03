package org.mensetsu.app.core.network

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class EndpointValidatorTest {

    @Test
    fun testValidHttpsUrl() {
        val res = EndpointValidator.validate("https://api.mensetsu.example.com")
        assertTrue(res is EndpointValidator.ValidationResult.Valid)
        assertEquals("https://api.mensetsu.example.com", (res as EndpointValidator.ValidationResult.Valid).normalizedUrl)
    }

    @Test
    fun testStripTrailingSlash() {
        val res = EndpointValidator.validate("https://api.mensetsu.example.com///")
        assertTrue(res is EndpointValidator.ValidationResult.Valid)
        assertEquals("https://api.mensetsu.example.com", (res as EndpointValidator.ValidationResult.Valid).normalizedUrl)
    }

    @Test
    fun testPermitLocalhostHttp() {
        val res1 = EndpointValidator.validate("http://localhost:3000")
        assertTrue(res1 is EndpointValidator.ValidationResult.Valid)
        assertEquals("http://localhost:3000", (res1 as EndpointValidator.ValidationResult.Valid).normalizedUrl)

        val res2 = EndpointValidator.validate("http://127.0.0.1:3000")
        assertTrue(res2 is EndpointValidator.ValidationResult.Valid)
        assertEquals("http://127.0.0.1:3000", (res2 as EndpointValidator.ValidationResult.Valid).normalizedUrl)

        val res3 = EndpointValidator.validate("http://10.0.2.2:3000")
        assertTrue(res3 is EndpointValidator.ValidationResult.Valid)
        assertEquals("http://10.0.2.2:3000", (res3 as EndpointValidator.ValidationResult.Valid).normalizedUrl)
    }

    @Test
    fun testRejectRemoteHttp() {
        val res = EndpointValidator.validate("http://api.remote.com:8080")
        assertTrue(res is EndpointValidator.ValidationResult.Invalid)
        assertTrue((res as EndpointValidator.ValidationResult.Invalid).reason.contains("HTTPS is required"))
        assertTrue(EndpointValidator.validate("http://[::1]:3000") is EndpointValidator.ValidationResult.Invalid)
        assertTrue(EndpointValidator.validate("http://app.localhost:3000") is EndpointValidator.ValidationResult.Invalid)
    }

    @Test
    fun testRejectInvalidOrEmptyUrl() {
        val resEmpty = EndpointValidator.validate("   ")
        assertTrue(resEmpty is EndpointValidator.ValidationResult.Invalid)

        val resNoScheme = EndpointValidator.validate("api.example.com")
        assertTrue(resNoScheme is EndpointValidator.ValidationResult.Invalid)

        val resFtp = EndpointValidator.validate("ftp://api.example.com")
        assertTrue(resFtp is EndpointValidator.ValidationResult.Invalid)
    }

    @Test
    fun testRejectUserInfoQueryFragmentAndBindAddress() {
        val resUserInfo = EndpointValidator.validate("https://user:pass@api.example.com")
        assertTrue(resUserInfo is EndpointValidator.ValidationResult.Invalid)
        assertTrue((resUserInfo as EndpointValidator.ValidationResult.Invalid).reason.contains("credentials or userinfo"))

        val resQuery = EndpointValidator.validate("https://api.example.com?foo=bar")
        assertTrue(resQuery is EndpointValidator.ValidationResult.Invalid)
        assertTrue((resQuery as EndpointValidator.ValidationResult.Invalid).reason.contains("query string"))

        assertTrue(EndpointValidator.validate("https://api.example.com?") is EndpointValidator.ValidationResult.Invalid)
        assertTrue(EndpointValidator.validate("https://api.example.com/api/v1") is EndpointValidator.ValidationResult.Invalid)

        val resFragment = EndpointValidator.validate("https://api.example.com#section")
        assertTrue(resFragment is EndpointValidator.ValidationResult.Invalid)
        assertTrue((resFragment as EndpointValidator.ValidationResult.Invalid).reason.contains("fragment"))

        val resBindAddr = EndpointValidator.validate("http://0.0.0.0:3000")
        assertTrue(resBindAddr is EndpointValidator.ValidationResult.Invalid)
        assertTrue((resBindAddr as EndpointValidator.ValidationResult.Invalid).reason.contains("0.0.0.0 is a bind address"))
    }
}
