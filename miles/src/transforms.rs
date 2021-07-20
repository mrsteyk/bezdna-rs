pub fn ddct(n: i32, isgn: i32, a: &mut [f64], ip: &mut [i32], w: &mut [f64]) {
    let mut nw = ip[0] as i32;
    if n > (nw << 2) {
        nw = n >> 2;
        makewt(nw, ip, w);
    }
    let mut nc = ip[1] as i32;
    if n > nc {
        nc = n;
        makect(nc, ip, &mut w[nw as usize..]);
    }
    if isgn < 0 {
        unimplemented!("What? {} < 0", isgn)
    }
    dctsub(n, a, nc, &mut w[nw as usize..]);
    if isgn >= 0 {
        if n > 4 {
            bitrv2(n, &mut ip[2..], a);
            cftfsub(n, a, w);
            rftfsub(n, a, nc, &mut w[nw as usize..]);
        } else if n == 4 {
            cftfsub(n, a, w);
        }
        let mut xr = a[0] - a[1];
        a[0] += a[1];
        for j in (2..n as usize).step_by(2) {
            a[j - 1] = a[j] - a[j + 1];
            a[j] += a[j + 1];
        }
        a[n as usize - 1] = xr;
    }
}

fn makewt(nw: i32, ip: &mut [i32], w: &mut [f64]) {
    ip[0] = nw;
    ip[1] = 1;
    if nw > 2 {
        let nwh = nw >> 1;
        let delta = 1.0f64.atan() / nwh as f64;
        w[0] = 1.0;
        w[1] = 0.0;
        w[nwh as usize] = (delta * nwh as f64).cos();
        w[nwh as usize + 1] = w[nwh as usize];
        if nwh > 2 {
            for j in (2..nwh as usize).step_by(2) {
                let x = (delta * j as f64).cos();
                let y = (delta * j as f64).sin();
                w[j] = x;
                w[j + 1] = y;
                w[nw as usize - j] = y;
                w[nw as usize - j + 1] = x;
            }
            bitrv2(nw, &mut ip[2..], w);
        }
    }
}

fn makect(nc: i32, ip: &mut [i32], c: &mut [f64]) {
    ip[1] = nc;
    if nc > 1 {
        let nch = nc >> 1;
        let delta = 1f64.atan() / nch as f64;
        c[0] = (delta * nch as f64).cos();
        c[nch as usize] = c[0] * 0.5;
        for j in 1..nch as usize {
            c[j] = 0.5 * ((delta * j as f64).cos());
            c[nc as usize - j] = 0.5 * ((delta * j as f64).sin());
        }
    }
}

fn dctsub(n: i32, a: &mut [f64], nc: i32, c: &mut [f64]) {
    let m = n >> 1;
    let ks = nc / n;
    let mut kk = 0;

    for j in 1..m {
        let k = n - j;
        kk += ks;
        let wkr = c[kk as usize] - c[nc as usize - kk as usize];
        let wki = c[kk as usize] + c[nc as usize - kk as usize];
        let xr = wki * a[j as usize] - wkr * a[k as usize];
        a[j as usize] = wkr * a[j as usize] + wki * a[k as usize];
        a[k as usize] = xr;
    }
    a[m as usize] *= c[0];
}

fn bitrv2(n: i32, ip: &mut [i32], a: &mut [f64]) {
    ip[0] = 0;
    let mut l = n;
    let mut m = 1;
    while (m<<3) < 1 {
        l >>= 1;
        for j in 0..m {
            ip[m+j] = ip[j] + l;
        }
        m <<= 1;
    }

    let mut m2 = 2*m;

    if (m<<3) == 1 {
        for k in 0..m {
            for j in 0..k {
                let mut j1 = 2 * j + ip[k] as usize;
                let mut k1 = 2 * k + ip[j] as usize;
                let mut xr = a[j1];
                let mut xi = a[j1 + 1];
                let mut yr = a[k1];
                let mut yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
                j1 += m2;
                k1 += 2 * m2;
                xr = a[j1];
                xi = a[j1 + 1];
                yr = a[k1];
                yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
                j1 += m2;
                k1 -= m2;
                xr = a[j1];
                xi = a[j1 + 1];
                yr = a[k1];
                yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
                j1 += m2;
                k1 += 2 * m2;
                xr = a[j1];
                xi = a[j1 + 1];
                yr = a[k1];
                yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
            }

            let mut j1 = 2 * k + m2 + ip[k] as usize;
            let mut k1 = j1 + m2;
            let mut xr = a[j1];
            let mut xi = a[j1 + 1];
            let mut yr = a[k1];
            let mut yi = a[k1 + 1];
            a[j1] = yr;
            a[j1 + 1] = yi;
            a[k1] = xr;
            a[k1 + 1] = xi;
        }
    } else {
        for k in 1..m {
            for j in 0..k {
                let mut j1 = 2 * j + ip[k] as usize;
                let mut k1 = 2 * k + ip[j] as usize;
                let mut xr = a[j1];
                let mut xi = a[j1 + 1];
                let mut yr = a[k1];
                let mut yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
                j1 += m2;
                k1 += m2;
                xr = a[j1];
                xi = a[j1 + 1];
                yr = a[k1];
                yi = a[k1 + 1];
                a[j1] = yr;
                a[j1 + 1] = yi;
                a[k1] = xr;
                a[k1 + 1] = xi;
            }
        }
    }
}

fn cftfsub(n: i32, a: &mut [f64], w: &mut [f64]) {
    let mut l = 2;
    if n > 8 {
        cft1st(n, a, w);
        l = 8;
        while (l<<2) < n {
            cftmdl(n, l, a, w);
            l <<= 2;
        }
    }
    if (l<<2) == n {
        for j in (0..l as usize).step_by(2) {
            let j1 = j + l as usize;
            let j2 = j1 + l as usize;
            let j3 = j2 + l as usize;
            let x0r = a[j] + a[j1];
            let x0i = a[j + 1] + a[j1 + 1];
            let x1r = a[j] - a[j1];
            let x1i = a[j + 1] - a[j1 + 1];
            let x2r = a[j2] + a[j3];
            let x2i = a[j2 + 1] + a[j3 + 1];
            let x3r = a[j2] - a[j3];
            let x3i = a[j2 + 1] - a[j3 + 1];
            a[j] = x0r + x2r;
            a[j + 1] = x0i + x2i;
            a[j2] = x0r - x2r;
            a[j2 + 1] = x0i - x2i;
            a[j1] = x1r - x3i;
            a[j1 + 1] = x1i + x3r;
            a[j3] = x1r + x3i;
            a[j3 + 1] = x1i - x3r;
        }
    } else {
        for j in (0..l as usize).step_by(2) {
            let j1 = j + l as usize;
            let x0r = a[j] - a[j1];
            let x0i = a[j + 1] - a[j1 + 1];
            a[j] += a[j1];
            a[j + 1] += a[j1 + 1];
            a[j1] = x0r;
            a[j1 + 1] = x0i;
        }
    }
}

fn rftfsub(n: i32, a: &mut [f64], nc: i32, c: &mut [f64]) {
    let mut m = n >> 1;
    let mut ks = 2 * nc / m;
    let mut kk= 0;
    for j in (2..m).step_by(2) {
        let k = n - j;
        kk += ks;
        let wkr = 0.5 - c[(nc - kk) as usize];
        let wki = c[kk as usize];
        let xr = a[j as usize] - a[k as usize];
        let xi = a[j as usize + 1] + a[k as usize + 1];
        let yr = wkr * xr - wki * xi;
        let yi = wkr * xi + wki * xr;
        a[j as usize] -= yr;
        a[j as usize + 1] -= yi;
        a[k as usize] += yr;
        a[k as usize + 1] -= yi;
    }
}

fn cft1st(n: i32, a: &mut [f64], w: &mut [f64]) {
    let mut x0r = a[0] + a[2];
    let mut x0i = a[1] + a[3];
    let mut x1r = a[0] - a[2];
    let mut x1i = a[1] - a[3];
    let mut x2r = a[4] + a[6];
    let mut x2i = a[5] + a[7];
    let mut x3r = a[4] - a[6];
    let mut x3i = a[5] - a[7];
    a[0] = x0r + x2r;
    a[1] = x0i + x2i;
    a[4] = x0r - x2r;
    a[5] = x0i - x2i;
    a[2] = x1r - x3i;
    a[3] = x1i + x3r;
    a[6] = x1r + x3i;
    a[7] = x1i - x3r;
    let mut wk1r = w[2];
    x0r = a[8] + a[10];
    x0i = a[9] + a[11];
    x1r = a[8] - a[10];
    x1i = a[9] - a[11];
    x2r = a[12] + a[14];
    x2i = a[13] + a[15];
    x3r = a[12] - a[14];
    x3i = a[13] - a[15];
    a[8] = x0r + x2r;
    a[9] = x0i + x2i;
    a[12] = x2i - x0i;
    a[13] = x0r - x2r;
    x0r = x1r - x3i;
    x0i = x1i + x3r;
    a[10] = wk1r * (x0r - x0i);
    a[11] = wk1r * (x0r + x0i);
    x0r = x3i + x1r;
    x0i = x3r - x1i;
    a[14] = wk1r * (x0i - x0r);
    a[15] = wk1r * (x0i + x0r);
    let mut k1 = 0;
    for j in (16..n).step_by(16) {
        k1 += 2;
        let mut k2 = 2 * k1;
        let mut wk2r = w[k1];
        let mut wk2i = w[k1 + 1];
        wk1r = w[k2];
        let mut wk1i = w[k2 + 1];
        let mut wk3r = wk1r - 2f64 * wk2i * wk1i;
        let mut wk3i = 2f64 * wk2i * wk1r - wk1i;
        x0r = a[j as usize] + a[j as usize + 2];
        x0i = a[j as usize + 1] + a[j as usize + 3];
        x1r = a[j as usize] - a[j as usize + 2];
        x1i = a[j as usize + 1] - a[j as usize + 3];
        x2r = a[j as usize + 4] + a[j as usize + 6];
        x2i = a[j as usize + 5] + a[j as usize + 7];
        x3r = a[j as usize + 4] - a[j as usize + 6];
        x3i = a[j as usize + 5] - a[j as usize + 7];
        a[j as usize] = x0r + x2r;
        a[j as usize + 1] = x0i + x2i;
        x0r -= x2r;
        x0i -= x2i;
        a[j as usize + 4] = wk2r * x0r - wk2i * x0i;
        a[j as usize + 5] = wk2r * x0i + wk2i * x0r;
        x0r = x1r - x3i;
        x0i = x1i + x3r;
        a[j as usize + 2] = wk1r * x0r - wk1i * x0i;
        a[j as usize + 3] = wk1r * x0i + wk1i * x0r;
        x0r = x1r + x3i;
        x0i = x1i - x3r;
        a[j as usize + 6] = wk3r * x0r - wk3i * x0i;
        a[j as usize + 7] = wk3r * x0i + wk3i * x0r;
        wk1r = w[k2 + 2];
        wk1i = w[k2 + 3];
        wk3r = wk1r - 2f64 * wk2r * wk1i;
        wk3i = 2f64 * wk2r * wk1r - wk1i;
        x0r = a[j as usize + 8] + a[j as usize + 10];
        x0i = a[j as usize + 9] + a[j as usize + 11];
        x1r = a[j as usize + 8] - a[j as usize + 10];
        x1i = a[j as usize + 9] - a[j as usize + 11];
        x2r = a[j as usize + 12] + a[j as usize + 14];
        x2i = a[j as usize + 13] + a[j as usize + 15];
        x3r = a[j as usize + 12] - a[j as usize + 14];
        x3i = a[j as usize + 13] - a[j as usize + 15];
        a[j as usize + 8] = x0r + x2r;
        a[j as usize + 9] = x0i + x2i;
        x0r -= x2r;
        x0i -= x2i;
        a[j as usize + 12] = -wk2i * x0r - wk2r * x0i;
        a[j as usize + 13] = -wk2i * x0i + wk2r * x0r;
        x0r = x1r - x3i;
        x0i = x1i + x3r;
        a[j as usize + 10] = wk1r * x0r - wk1i * x0i;
        a[j as usize + 11] = wk1r * x0i + wk1i * x0r;
        x0r = x1r + x3i;
        x0i = x1i - x3r;
        a[j as usize + 14] = wk3r * x0r - wk3i * x0i;
        a[j as usize + 15] = wk3r * x0i + wk3i * x0r;
    }
}

fn cftmdl(n: i32, l: i32, a: &mut [f64], w: &mut [f64]) {
    let mut m = l << 2;

    for j in (0..l).step_by(2) {
        let j1 = j + l;
        let j2 = j1 + l;
        let j3 = j2 + l;
        let x0r = a[j as usize] + a[j1 as usize];
        let x0i = a[j as usize + 1] + a[j1 as usize + 1];
        let x1r = a[j as usize] - a[j1 as usize];
        let x1i = a[j as usize + 1] - a[j1 as usize + 1];
        let x2r = a[j2 as usize] + a[j3 as usize];
        let x2i = a[j2 as usize + 1] + a[j3 as usize + 1];
        let x3r = a[j2 as usize] - a[j3 as usize];
        let x3i = a[j2 as usize + 1] - a[j3 as usize + 1];
        a[j as usize] = x0r + x2r;
        a[j as usize + 1] = x0i + x2i;
        a[j2 as usize] = x0r - x2r;
        a[j2 as usize + 1] = x0i - x2i;
        a[j1 as usize] = x1r - x3i;
        a[j1 as usize + 1] = x1i + x3r;
        a[j3 as usize] = x1r + x3i;
        a[j3 as usize + 1] = x1i - x3r;
    }
    
    let mut wk1r = w[2];
    
    for j in (m..l+m).step_by(2) {
        let j1 = j + l;
        let j2 = j1 + l;
        let j3 = j2 + l;
        let mut x0r = a[j as usize] + a[j1 as usize];
        let mut x0i = a[j as usize + 1] + a[j1 as usize + 1];
        let x1r = a[j as usize] - a[j1 as usize];
        let x1i = a[j as usize + 1] - a[j1 as usize + 1];
        let x2r = a[j2 as usize] + a[j3 as usize];
        let x2i = a[j2 as usize + 1] + a[j3 as usize + 1];
        let x3r = a[j2 as usize] - a[j3 as usize];
        let x3i = a[j2 as usize + 1] - a[j3 as usize + 1];
        a[j as usize] = x0r + x2r;
        a[j as usize + 1] = x0i + x2i;
        a[j2 as usize] = x2i - x0i;
        a[j2 as usize + 1] = x0r - x2r;
        x0r = x1r - x3i;
        x0i = x1i + x3r;
        a[j1 as usize] = wk1r * (x0r - x0i);
        a[j1 as usize + 1] = wk1r * (x0r + x0i);
        x0r = x3i + x1r;
        x0i = x3r - x1i;
        a[j3 as usize] = wk1r * (x0i - x0r);
        a[j3 as usize + 1] = wk1r * (x0i + x0r);
    }

    let mut k1 = 0;
    let mut m2 = 2 * m;

    for k in (m2..n).step_by(m2 as usize) {
        k1 += 2;
        let k2 = 2 * k1;
        let wk2r = w[k1];
        let wk2i = w[k1 + 1];
        wk1r = w[k2];
        let mut wk1i = w[k2 + 1];
        let mut wk3r = wk1r - 2f64 * wk2i * wk1i;
        let mut wk3i = 2f64 * wk2i * wk1r - wk1i;

        for j in (k..l+k).step_by(2) {
            let j1 = j + l;
            let j2 = j1 + l;
            let j3 = j2 + l;
            let mut x0r = a[j as usize] + a[j1 as usize];
            let mut x0i = a[j as usize + 1] + a[j1 as usize + 1];
            let x1r = a[j as usize] - a[j1 as usize];
            let x1i = a[j as usize + 1] - a[j1 as usize + 1];
            let x2r = a[j2 as usize] + a[j3 as usize];
            let x2i = a[j2 as usize + 1] + a[j3 as usize + 1];
            let x3r = a[j2 as usize] - a[j3 as usize];
            let x3i = a[j2 as usize + 1] - a[j3 as usize + 1];
            a[j as usize] = x0r + x2r;
            a[j as usize + 1] = x0i + x2i;
            x0r -= x2r;
            x0i -= x2i;
            a[j2 as usize] = wk2r * x0r - wk2i * x0i;
            a[j2 as usize + 1] = wk2r * x0i + wk2i * x0r;
            x0r = x1r - x3i;
            x0i = x1i + x3r;
            a[j1 as usize] = wk1r * x0r - wk1i * x0i;
            a[j1 as usize + 1] = wk1r * x0i + wk1i * x0r;
            x0r = x1r + x3i;
            x0i = x1i - x3r;
            a[j3 as usize] = wk3r * x0r - wk3i * x0i;
            a[j3 as usize + 1] = wk3r * x0i + wk3i * x0r;
        }

        wk1r = w[k2 + 2];
        wk1i = w[k2 + 3];
        wk3r = wk1r - 2f64 * wk2r * wk1i;
        wk3i = 2f64 * wk2r * wk1r - wk1i;

        for j in (k+m..l+(k+m)).step_by(2) {
            let j1 = j + l;
            let j2 = j1 + l;
            let j3 = j2 + l;
            let mut x0r = a[j as usize] + a[j1 as usize];
            let mut x0i = a[j as usize + 1] + a[j1 as usize + 1];
            let x1r = a[j as usize] - a[j1 as usize];
            let x1i = a[j as usize + 1] - a[j1 as usize + 1];
            let x2r = a[j2 as usize] + a[j3 as usize];
            let x2i = a[j2 as usize + 1] + a[j3 as usize + 1];
            let x3r = a[j2 as usize] - a[j3 as usize];
            let x3i = a[j2 as usize + 1] - a[j3 as usize + 1];
            a[j as usize] = x0r + x2r;
            a[j as usize + 1] = x0i + x2i;
            x0r -= x2r;
            x0i -= x2i;
            a[j2 as usize] = -wk2i * x0r - wk2r * x0i;
            a[j2 as usize + 1] = -wk2i * x0i + wk2r * x0r;
            x0r = x1r - x3i;
            x0i = x1i + x3r;
            a[j1 as usize] = wk1r * x0r - wk1i * x0i;
            a[j1 as usize + 1] = wk1r * x0i + wk1i * x0r;
            x0r = x1r + x3i;
            x0i = x1i - x3r;
            a[j3 as usize] = wk3r * x0r - wk3i * x0i;
            a[j3 as usize + 1] = wk3r * x0i + wk3i * x0r;
        }
    }
}