<?php

class HttpClient
{
    private static function getCurlOptions(string $url): array
    {
        return [
            CURLOPT_URL => $url,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_MAXREDIRS => 3,
            CURLOPT_TIMEOUT => 10,
            CURLOPT_USERAGENT => 'Mozilla/5.0 (compatible; HTML-Minifier/1.0)',
            CURLOPT_HTTPHEADER => [
                'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
                'Accept-Language: en-US,en;q=0.5',
                'Accept-Encoding: identity',
                'Connection: keep-alive',
            ],
            CURLOPT_SSL_VERIFYPEER => false,
            CURLOPT_HEADER => false,
        ];
    }

    public static function fetch(string $url): HttpResponse
    {
        $curl = curl_init();
        curl_setopt_array($curl, self::getCurlOptions($url));

        $startTime = microtime(true);
        $html = curl_exec($curl);
        $endTime = microtime(true);

        $httpCode = curl_getinfo($curl, CURLINFO_HTTP_CODE);
        $contentType = curl_getinfo($curl, CURLINFO_CONTENT_TYPE);
        $error = curl_error($curl);

        curl_close($curl);

        $fetchTime = ($endTime - $startTime) * 1000;

        return new HttpResponse($html, $httpCode, $contentType, $error, $fetchTime);
    }
}