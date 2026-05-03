import requests
from bs4 import BeautifulSoup
import json
import os
import re
from datetime import datetime

def clean_text(text):
    """Removes extra whitespace and newlines for cleaner indexing."""
    return re.sub(r'\s+', ' ', text).strip()

def run_crawler(urls):
    search_index = []
    
    # Modern headers to mimic a real browser
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36',
        'Accept-Language': 'en-US,en;q=0.9',
    }

    for url in urls:
        if not url.startswith('http'):
            continue
            
        try:
            print(f"🔍 Indexing: {url}")
            response = requests.get(url, headers=headers, timeout=15)
            response.raise_for_status()
            
            # Parse HTML
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # 1. Extract Title
            page_title = soup.title.string.strip() if soup.title else url
            
            # 2. Extract Snippet (Description)
            description = ""
            meta_desc = soup.find('meta', attrs={'name': 'description'}) or \
                        soup.find('meta', attrs={'property': 'og:description'})
            
            if meta_desc and meta_desc.get('content'):
                description = meta_desc['content'].strip()
            else:
                # Fallback: Scrape the first 200 chars of main text
                # Deleting non-content tags first
                for junk in soup(["script", "style", "nav", "footer", "header", "aside"]):
                    junk.decompose()
                description = clean_text(soup.get_text())[:200] + "..."

            # 3. Extract Full Content for Search Matching
            # We index a larger chunk of text so users can find pages via keywords
            full_content = clean_text(soup.get_text()).lower()
            
            search_index.append({
                "title": page_title,
                "url": url,
                "snippet": description,
                "content": full_content[:10000], # Indexing first 10k chars
                "last_updated": datetime.utcnow().isoformat()
            })

        except Exception as e:
            print(f"⚠️ Error crawling {url}: {e}")

    # Write results to the JSON file used by index.html
    with open('search_index.json', 'w', encoding='utf-8') as f:
        json.dump(search_index, f, indent=4, ensure_ascii=False)
        
    print(f"\n✅ Build Complete. {len(search_index)} pages stored in search_index.json")

if __name__ == "__main__":
    # --- CONFIGURE YOUR SEED LIST HERE ---
    # These are the URLs your search engine will 'own'
    target_sites = [
        "https://en.wikipedia.org/wiki/Web_crawler",
        "https://www.python.org/doc/",
        "https://github.com/about",
        "https://developer.mozilla.org/en-US/docs/Web/Guide",
        "https://www.w3schools.com/html/default.asp"
    ]
    
    run_crawler(target_sites)
