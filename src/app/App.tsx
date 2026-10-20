import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Product = {
  key: string;
  name: string;
  available: boolean;
  launchPath: string;
};

export default function App() {
  const [products, setProducts] = useState<Product[]>([]);
  const [status, setStatus] = useState("Select a product");
  async function refresh() {
    try {
      setProducts(await invoke<Product[]>("list_products"));
    } catch (error) {
      setStatus(String(error));
    }
  }
  useEffect(() => {
    void refresh();
  }, []);
  async function launch(key: string) {
    setStatus("Launching…");
    try {
      await invoke("launch_product", { productKey: key });
    } catch (error) {
      setStatus(String(error));
    }
  }
  return (
    <main className="launcher-shell">
      <section className="launcher-card">
        <p className="launcher-eyebrow">Private product launcher</p>
        <h1>Overlay Forge</h1>
        <p>{status}</p>
        <div className="launcher-products">
          {products.map((product) => (
            <button
              disabled={!product.available}
              key={product.key}
              onClick={() => void launch(product.key)}
              type="button"
            >
              <strong>{product.name}</strong>
              <span>
                {product.available
                  ? "Launch"
                  : product.key === "journal"
                    ? "Start npm.cmd run dev in overlay-forge-journal"
                    : "Build or install this product first"}
              </span>
            </button>
          ))}
        </div>
        <button type="button" onClick={() => void refresh()}>
          Refresh products
        </button>
      </section>
    </main>
  );
}
